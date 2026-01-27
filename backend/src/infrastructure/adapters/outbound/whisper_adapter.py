"""Adapter: WhisperAdapter - Implementación del puerto SpeechRecognition con Whisper."""

import time
import asyncio
from pathlib import Path
from typing import Optional
from concurrent.futures import ThreadPoolExecutor

import numpy as np
import whisper
import torch

from ....domain import (
    AudioBuffer,
    Transcription,
    Language,
    SpeechRecognitionPort
)


class WhisperAdapter(SpeechRecognitionPort):
    """
    Adaptador de Whisper para reconocimiento de voz.

    Implementa el puerto SpeechRecognitionPort usando OpenAI Whisper.
    Optimizado para CPU con quantización int8.
    """

    SUPPORTED_LANGUAGES = [
        "es", "en", "pt", "fr", "de", "it", "ja", "ko", "zh", "ru"
    ]

    def __init__(
        self,
        model_name: str = "large-v3-turbo",
        models_path: Optional[Path] = None,
        device: str = "cpu",
        compute_type: str = "int8",
        num_workers: int = 2
    ):
        self._model_name = model_name
        self._models_path = models_path or Path("/app/models")
        self._device = device
        self._compute_type = compute_type
        self._model: Optional[whisper.Whisper] = None
        self._executor = ThreadPoolExecutor(max_workers=num_workers)
        self._is_loaded = False

    def load_model(self) -> None:
        """Carga el modelo Whisper."""
        if self._is_loaded:
            return

        print(f"Loading Whisper model '{self._model_name}' on {self._device}...")

        # Verificar si existe el modelo local
        local_model_path = self._models_path / f"{self._model_name}.pt"

        if local_model_path.exists():
            print(f"Loading from local path: {local_model_path}")
            self._model = whisper.load_model(
                str(local_model_path),
                device=self._device
            )
        else:
            print(f"Downloading model '{self._model_name}'...")
            self._model = whisper.load_model(
                self._model_name,
                device=self._device,
                download_root=str(self._models_path)
            )

        self._is_loaded = True
        print(f"Model loaded successfully on {self._device}")

    def _transcribe_sync(self, audio_buffer: AudioBuffer) -> Transcription:
        """Transcripción síncrona (para ejecutar en thread pool)."""
        if not self._is_loaded or self._model is None:
            raise RuntimeError("Model not loaded. Call load_model() first.")

        start_time = time.time()

        # Preparar audio
        audio = audio_buffer.data.astype(np.float32)

        # Transcribir con Whisper
        result = self._model.transcribe(
            audio,
            language=audio_buffer.language.code,
            fp16=False,  # CPU no soporta fp16
            verbose=False
        )

        processing_time_ms = int((time.time() - start_time) * 1000)

        # Crear transcripción desde resultado
        return Transcription.from_whisper_result(
            result=result,
            language=audio_buffer.language,
            audio_duration_ms=audio_buffer.duration_ms,
            processing_time_ms=processing_time_ms,
            audio_buffer_id=audio_buffer.id
        )

    async def transcribe(self, audio_buffer: AudioBuffer) -> Transcription:
        """
        Transcribe un buffer de audio a texto.

        Args:
            audio_buffer: Buffer de audio a transcribir.

        Returns:
            Transcription con el texto transcrito.
        """
        if not self._is_loaded:
            self.load_model()

        # Ejecutar transcripción en thread pool para no bloquear el event loop
        loop = asyncio.get_event_loop()
        transcription = await loop.run_in_executor(
            self._executor,
            self._transcribe_sync,
            audio_buffer
        )

        return transcription

    async def is_available(self) -> bool:
        """Verifica si el servicio está disponible."""
        return self._is_loaded and self._model is not None

    def get_supported_languages(self) -> list[str]:
        """Obtiene los idiomas soportados."""
        return self.SUPPORTED_LANGUAGES.copy()

    def get_model_info(self) -> dict:
        """Obtiene información del modelo cargado."""
        return {
            "name": self._model_name,
            "device": self._device,
            "compute_type": self._compute_type,
            "is_loaded": self._is_loaded,
            "models_path": str(self._models_path),
            "cuda_available": torch.cuda.is_available()
        }

    def shutdown(self) -> None:
        """Libera recursos."""
        self._executor.shutdown(wait=True)
        if self._model is not None:
            del self._model
            self._model = None
            self._is_loaded = False
            # Liberar memoria CUDA si estaba en uso
            if torch.cuda.is_available():
                torch.cuda.empty_cache()
