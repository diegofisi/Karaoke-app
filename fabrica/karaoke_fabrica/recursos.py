"""Detecta el equipo y elige los modelos que mejor rinden en el.

La app no pregunta: elige segun la GPU y la memoria. El reconocedor es
siempre el grande (1,7B): el de 0,6B dejaba el espanol cantado en 43,5 % de
inicios a <=100 ms frente a 77,8 % (eval 2026-10-16, perfil-medio-asr06).
Lo que cambia por equipo: con poca VRAM el reconocedor se libera antes de
alinear; sin GPU, separador ligero y todos los nucleos. `KARAOKE_PERFIL` (alto | medio | basico) fuerza los modelos de un
perfil, para medir con eval/ o para probar.
"""

from __future__ import annotations

import os
import platform
from dataclasses import dataclass
from functools import lru_cache

SEPARADOR_PRECISO = "vocals_mel_band_roformer.ckpt"   # Mel-RoFormer, SDR voz ~11
SEPARADOR_LIGERO = "Kim_Vocal_2.onnx"                  # MDX ONNX, SDR voz 10,2, rapido en CPU
ASR_GRANDE = "Qwen/Qwen3-ASR-1.7B"
ASR_PEQUENO = "Qwen/Qwen3-ASR-0.6B"

VRAM_MINIMA_GB = 6.0    # 1,7B en bf16 + alineador caben con margen
RAM_MAC_MINIMA_GB = 16.0


@dataclass(frozen=True)
class Perfil:
    nombre: str            # "alto" | "medio" | "basico"
    dispositivo: str       # "cuda" | "mps" | "cpu"
    separador: str
    asr: str
    descripcion: str


def _ram_gb() -> float:
    try:
        import psutil

        return psutil.virtual_memory().total / 1024**3
    except Exception:  # sin psutil: se asume poca memoria
        return 8.0


def _perfil(nombre: str, dispositivo: str, detalle: str) -> Perfil:
    if nombre == "alto":
        return Perfil("alto", dispositivo, SEPARADOR_PRECISO, ASR_GRANDE, f"Máxima precisión ({detalle})")
    if nombre == "medio":
        return Perfil("medio", dispositivo, SEPARADOR_PRECISO, ASR_GRANDE,
                      f"Máxima precisión, liberando memoria entre pasos ({detalle})")
    return Perfil("basico", dispositivo, SEPARADOR_LIGERO, ASR_GRANDE,
                  f"Separador ligero; procesar puede tardar varios minutos ({detalle})")


@lru_cache(maxsize=1)
def detectar() -> Perfil:
    import torch

    forzado = os.environ.get("KARAOKE_PERFIL", "").strip().lower()
    if torch.cuda.is_available():
        vram = torch.cuda.get_device_properties(0).total_memory / 1024**3
        gpu = torch.cuda.get_device_name(0)
        dispositivo, detalle = "cuda", f"{gpu}, {vram:.0f} GB"
        nombre = "alto" if vram >= VRAM_MINIMA_GB else "medio"
    elif getattr(torch.backends, "mps", None) is not None and torch.backends.mps.is_available():
        ram = _ram_gb()
        dispositivo, detalle = "mps", f"Apple {platform.machine()}, {ram:.0f} GB"
        nombre = "alto" if ram >= RAM_MAC_MINIMA_GB else "medio"
    else:
        nucleos = os.cpu_count() or 4
        torch.set_num_threads(nucleos)
        dispositivo, detalle = "cpu", f"sin GPU, {nucleos} núcleos"
        nombre = "basico"
    if forzado in ("alto", "medio", "basico"):
        nombre = forzado
    return _perfil(nombre, dispositivo, detalle)
