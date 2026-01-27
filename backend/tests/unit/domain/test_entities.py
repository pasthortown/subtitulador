"""Tests para Entidades del dominio."""

import pytest
import numpy as np

from src.domain.entities import AudioBuffer, Transcription
from src.domain.value_objects import Language, AudioFormat


class TestAudioBuffer:
    """Tests para la entidad AudioBuffer."""

    def test_create_empty_buffer(self):
        """Test crear buffer vacío."""
        buffer = AudioBuffer()
        assert buffer.is_empty
        assert buffer.duration_seconds == 0.0
        assert buffer.num_samples == 0

    def test_create_buffer_with_data(self):
        """Test crear buffer con datos."""
        data = np.zeros(16000, dtype=np.float32)  # 1 segundo a 16kHz
        buffer = AudioBuffer(data=data)

        assert not buffer.is_empty
        assert buffer.duration_seconds == 1.0
        assert buffer.duration_ms == 1000
        assert buffer.num_samples == 16000

    def test_buffer_normalizes_int16_data(self):
        """Test que el buffer normaliza datos int16."""
        # Simular datos int16 (rango -32768 a 32767)
        data = np.array([32767, -32768, 0], dtype=np.float32)
        buffer = AudioBuffer(data=data)

        # Debería normalizar a rango -1 a 1
        assert buffer.data.max() <= 1.0
        assert buffer.data.min() >= -1.0

    def test_buffer_calculates_rms_power(self):
        """Test cálculo de potencia RMS."""
        # Señal constante de 0.5
        data = np.full(1000, 0.5, dtype=np.float32)
        buffer = AudioBuffer(data=data)

        assert abs(buffer.rms_power - 0.5) < 0.001

    def test_is_valid_for_transcription(self):
        """Test validación para transcripción."""
        # Buffer muy corto
        short_data = np.zeros(100, dtype=np.float32)  # ~6ms
        short_buffer = AudioBuffer(data=short_data)
        assert not short_buffer.is_valid_for_transcription()

        # Buffer válido
        valid_data = np.zeros(8000, dtype=np.float32)  # 500ms
        valid_buffer = AudioBuffer(data=valid_data)
        assert valid_buffer.is_valid_for_transcription()

    def test_from_bytes(self):
        """Test crear buffer desde bytes."""
        original_data = np.array([0.1, 0.2, 0.3], dtype=np.float32)
        audio_bytes = original_data.tobytes()

        buffer = AudioBuffer.from_bytes(
            audio_bytes=audio_bytes,
            audio_format=AudioFormat.whisper_format()
        )

        np.testing.assert_array_almost_equal(buffer.data, original_data)


class TestTranscription:
    """Tests para la entidad Transcription."""

    def test_create_empty_transcription(self):
        """Test crear transcripción vacía."""
        trans = Transcription.empty()
        assert trans.is_empty
        assert trans.text == ""
        assert trans.word_count == 0

    def test_create_transcription_with_text(self):
        """Test crear transcripción con texto."""
        trans = Transcription(
            text="Hola mundo",
            language=Language.spanish(),
            confidence=0.95
        )

        assert not trans.is_empty
        assert trans.text == "Hola mundo"
        assert trans.word_count == 2
        assert trans.char_count == 10

    def test_transcription_strips_whitespace(self):
        """Test que la transcripción limpia espacios."""
        trans = Transcription(text="  Hola mundo  ")
        assert trans.text == "Hola mundo"

    def test_confidence_is_clamped(self):
        """Test que la confianza se limita a 0-1."""
        trans1 = Transcription(text="test", confidence=1.5)
        assert trans1.confidence == 1.0

        trans2 = Transcription(text="test", confidence=-0.5)
        assert trans2.confidence == 0.0

    def test_processing_ratio(self):
        """Test cálculo de ratio de procesamiento."""
        trans = Transcription(
            text="test",
            audio_duration_ms=1000,
            processing_time_ms=500
        )
        assert trans.processing_ratio == 0.5
        assert trans.is_realtime

        slow_trans = Transcription(
            text="test",
            audio_duration_ms=1000,
            processing_time_ms=2000
        )
        assert slow_trans.processing_ratio == 2.0
        assert not slow_trans.is_realtime

    def test_transcription_bool(self):
        """Test conversión a bool."""
        empty = Transcription(text="")
        with_text = Transcription(text="Hello")

        assert not bool(empty)
        assert bool(with_text)

    def test_from_whisper_result(self):
        """Test crear desde resultado de Whisper."""
        whisper_result = {
            "text": "Hola mundo",
            "segments": [
                {"avg_logprob": -0.5}
            ]
        }

        trans = Transcription.from_whisper_result(
            result=whisper_result,
            language=Language.spanish(),
            audio_duration_ms=1000,
            processing_time_ms=200
        )

        assert trans.text == "Hola mundo"
        assert trans.language == Language.spanish()
        assert trans.audio_duration_ms == 1000
        assert trans.processing_time_ms == 200
        assert 0 <= trans.confidence <= 1
