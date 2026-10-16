"""Decide donde corre cada modelo segun la memoria LIBRE en ese momento.

No hay perfiles por tipo de equipo: antes de cargar cada modelo se mira cuanta
memoria queda libre ahora mismo en la GPU (CUDA o la GPU de Apple) y en la RAM,
y se compara con lo que ese modelo necesita (medido el 2026-10-16, bfloat16):
si cabe en la GPU va a la GPU; si no, a la CPU; si tampoco hay RAM, se avisa
de que ira lento. Asi un equipo con poca VRAM usa la GPU para unas cosas y la
CPU para otras, y uno con otros programas abiertos no se ahoga.

El reconocedor es siempre el grande (1,7B): el de 0,6B dejaba el espanol
cantado en 43,5 % de inicios a <=100 ms frente a 77,8 % (eval perfil-medio-asr06).
`KARAOKE_SIN_GPU=1` fuerza todo a la CPU (para probar).
"""

from __future__ import annotations

import os
import platform
from dataclasses import dataclass

from .eventos import aviso, log

SEPARADOR_PRECISO = "vocals_mel_band_roformer.ckpt"   # Mel-RoFormer, SDR voz ~11; lento en CPU
SEPARADOR_LIGERO = "Kim_Vocal_2.onnx"                  # MDX ONNX, SDR voz 10,2; rapido en CPU
ASR_GRANDE = "Qwen/Qwen3-ASR-1.7B"

# Pico de memoria de cada modelo (GB), medido con torch en GPU en bfloat16.
NECESITA_GB = {
    "separador": 2.0,
    "reconocedor": 5.8,
    "alineador": 1.9,
    "mms": 1.8,
}
MARGEN_GB = 0.7  # para el audio, las activaciones y lo que otros programas pidan mientras


def _sin_gpu_forzado() -> bool:
    return os.environ.get("KARAOKE_SIN_GPU", "").strip() in ("1", "true", "si")


def tipo_gpu() -> str | None:
    """"cuda", "mps" o None."""
    import torch

    if _sin_gpu_forzado():
        return None
    if torch.cuda.is_available():
        return "cuda"
    if getattr(torch.backends, "mps", None) is not None and torch.backends.mps.is_available():
        return "mps"
    return None


def gpu_libre_gb() -> float:
    """Memoria de GPU libre ahora mismo (0 si no hay GPU)."""
    import torch

    tipo = tipo_gpu()
    if tipo == "cuda":
        libre, _ = torch.cuda.mem_get_info()
        return libre / 1024**3
    if tipo == "mps":
        # Memoria unificada: lo que Metal recomienda usar menos lo que ya usa.
        maximo = torch.mps.recommended_max_memory()
        return max(0.0, (maximo - torch.mps.driver_allocated_memory()) / 1024**3)
    return 0.0


def ram_libre_gb() -> float:
    try:
        import psutil

        return psutil.virtual_memory().available / 1024**3
    except Exception:  # sin psutil: se asume poca memoria
        return 4.0


def liberar_gpu() -> None:
    import gc

    import torch

    gc.collect()
    if torch.cuda.is_available():
        torch.cuda.empty_cache()
    elif tipo_gpu() == "mps":
        torch.mps.empty_cache()


def dispositivo_para(modelo: str):
    """(dispositivo, dtype) para `modelo` segun la memoria libre ahora mismo."""
    import torch

    necesita = NECESITA_GB[modelo] + MARGEN_GB
    gpu = tipo_gpu()
    if gpu is not None:
        liberar_gpu()
        libre = gpu_libre_gb()
        if libre >= necesita:
            log.info("%s en %s (%.1f GB libres, necesita %.1f)", modelo, gpu, libre, necesita)
            return ("cuda:0" if gpu == "cuda" else "mps"), torch.bfloat16
        log.info("%s no cabe en la GPU (%.1f GB libres, necesita %.1f): va a la CPU", modelo, libre, necesita)
    else:
        torch.set_num_threads(os.cpu_count() or 4)
    ram = ram_libre_gb()
    if ram < necesita:
        aviso(f"Memoria justa para {modelo} ({ram:.1f} GB libres de RAM): puede ir muy lento. "
              "Cierra otros programas si tarda demasiado.")
    # bfloat16 tambien en CPU: en float32 el reconocedor ocupaba 12 GB de RAM.
    return "cpu", torch.bfloat16


def separador_para_ahora() -> str:
    """Mel-RoFormer si cabe en la GPU; si no, el ligero (en CPU RoFormer tarda mucho)."""
    if tipo_gpu() is not None:
        liberar_gpu()
        if gpu_libre_gb() >= NECESITA_GB["separador"] + MARGEN_GB:
            return SEPARADOR_PRECISO
    return SEPARADOR_LIGERO


@dataclass(frozen=True)
class Resumen:
    """Para el diagnostico: que hay ahora y que se usaria."""
    descripcion: str
    separador: str


def resumen() -> Resumen:
    import torch

    gpu = tipo_gpu()
    ram = ram_libre_gb()
    separador = separador_para_ahora()
    if gpu == "cuda":
        libre, total = (x / 1024**3 for x in torch.cuda.mem_get_info())
        nombre = f"{torch.cuda.get_device_name(0)}: {libre:.1f} de {total:.0f} GB libres"
    elif gpu == "mps":
        nombre = f"GPU de Apple {platform.machine()}: {gpu_libre_gb():.1f} GB libres"
    else:
        nombre = f"sin GPU, {os.cpu_count() or 4} núcleos"
    todo = sum(NECESITA_GB[m] for m in ("reconocedor", "alineador", "mms")) + MARGEN_GB
    if gpu is not None and gpu_libre_gb() >= todo:
        plan = "todo en la GPU"
    elif gpu is not None and gpu_libre_gb() >= NECESITA_GB["alineador"] + MARGEN_GB:
        plan = "GPU y CPU según la memoria libre en cada paso"
    else:
        plan = "en la CPU; procesar puede tardar varios minutos"
    return Resumen(f"{nombre}; RAM libre {ram:.1f} GB. Plan: {plan}", separador)
