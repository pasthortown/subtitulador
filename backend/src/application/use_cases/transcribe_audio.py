"""Use Case: TranscribeAudio - Caso de uso para transcribir audio."""

import time
import base64
from typing import AsyncGenerator

import numpy as np

from ...domain import (
    AudioBuffer,
    Transcription,
    Language,
    AudioFormat,
    TranscriptionService,
    SpeechRecognitionPort
)
from ..dtos import TranscriptionRequest, TranscriptionResponse


class TranscribeAudioUseCase:
    """
    Caso de uso para transcribir audio.

    Orquesta la validación, procesamiento y transcripción de audio.
    """

    def __init__(
        self,
        speech_recognition: SpeechRecognitionPort,
        transcription_service: TranscriptionService
    ):
        self._speech_recognition = speech_recognition
        self._transcription_service = transcription_service

    async def execute(self, request: TranscriptionRequest) -> TranscriptionResponse:
        """
        Ejecuta la transcripción de audio.

        Args:
            request: DTO con los datos de la solicitud.

        Returns:
            TranscriptionResponse con el resultado.
        """
        start_time = time.time()

        try:
            # Decodificar audio base64
            audio_bytes = base64.b64decode(request.audio)
            audio_data = np.frombuffer(audio_bytes, dtype=np.float32)

            # Crear buffer de audio
            language = Language(request.language)
            audio_format = AudioFormat.whisper_format()
            audio_buffer = AudioBuffer(
                data=audio_data,
                audio_format=audio_format,
                language=language
            )

            # Validar buffer
            validation_errors = self._transcription_service.validate_audio_buffer(
                audio_buffer
            )
            if validation_errors:
                return TranscriptionResponse.failure(
                    code="INVALID_AUDIO",
                    message="Audio validation failed",
                    details={"errors": validation_errors}
                )

            # Transcribir
            transcription = await self._speech_recognition.transcribe(audio_buffer)

            # Calcular tiempo de procesamiento
            processing_time_ms = int((time.time() - start_time) * 1000)

            # Normalizar texto
            normalized_text = self._transcription_service.normalize_transcription_text(
                transcription.text
            )

            return TranscriptionResponse.ok(
                text=normalized_text,
                language=transcription.language.code,
                confidence=transcription.confidence,
                duration_ms=audio_buffer.duration_ms,
                processing_time_ms=processing_time_ms
            )

        except ValueError as e:
            return TranscriptionResponse.failure(
                code="VALIDATION_ERROR",
                message=str(e)
            )
        except Exception as e:
            return TranscriptionResponse.failure(
                code="TRANSCRIPTION_ERROR",
                message=f"Transcription failed: {str(e)}"
            )

    async def execute_stream(
        self,
        audio_chunks: AsyncGenerator[bytes, None],
        language: str
    ) -> AsyncGenerator[TranscriptionResponse, None]:
        """
        Ejecuta transcripción en streaming.

        Args:
            audio_chunks: Generador de chunks de audio.
            language: Código de idioma.

        Yields:
            TranscriptionResponse parciales.
        """
        buffer = []
        lang = Language(language)
        audio_format = AudioFormat.whisper_format()

        async for chunk in audio_chunks:
            # Acumular chunks
            audio_data = np.frombuffer(chunk, dtype=np.float32)
            buffer.extend(audio_data)

            # Procesar cada 0.5 segundos de audio
            min_samples = audio_format.sample_rate // 2
            if len(buffer) >= min_samples:
                audio_buffer = AudioBuffer(
                    data=np.array(buffer, dtype=np.float32),
                    audio_format=audio_format,
                    language=lang
                )

                start_time = time.time()
                transcription = await self._speech_recognition.transcribe(
                    audio_buffer
                )
                processing_time_ms = int((time.time() - start_time) * 1000)

                if transcription.text:
                    yield TranscriptionResponse.ok(
                        text=transcription.text,
                        language=lang.code,
                        confidence=transcription.confidence,
                        duration_ms=audio_buffer.duration_ms,
                        processing_time_ms=processing_time_ms
                    )

                # Mantener un pequeño solapamiento para continuidad
                overlap = audio_format.sample_rate // 10
                buffer = buffer[-overlap:] if len(buffer) > overlap else []
