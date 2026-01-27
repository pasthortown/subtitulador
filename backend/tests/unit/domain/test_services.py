"""Tests for domain services."""

import pytest
import numpy as np

from src.domain.services import TranscriptionService
from src.domain.entities import AudioBuffer, Transcription
from src.domain.value_objects import Language, AudioFormat


class TestTranscriptionService:
    """Tests for TranscriptionService."""

    @pytest.fixture
    def service(self):
        """Create transcription service instance."""
        return TranscriptionService()

    @pytest.fixture
    def valid_audio_buffer(self):
        """Create a valid audio buffer for testing."""
        # 1 second of audio at 16kHz = 16000 samples
        samples = 16000
        audio_data = np.zeros(samples, dtype=np.float32)
        return AudioBuffer(
            data=audio_data,
            audio_format=AudioFormat.whisper_format(),
            language=Language("es")
        )

    def test_validate_valid_audio_buffer(self, service, valid_audio_buffer):
        """Test validating a valid audio buffer."""
        errors = service.validate_audio_buffer(valid_audio_buffer)
        assert errors == []

    def test_validate_empty_audio_buffer(self, service):
        """Test validating an empty audio buffer."""
        audio_buffer = AudioBuffer(
            data=np.array([], dtype=np.float32),
            audio_format=AudioFormat.whisper_format(),
            language=Language("es")
        )
        errors = service.validate_audio_buffer(audio_buffer)
        assert "Audio buffer is empty" in errors

    def test_validate_audio_too_short(self, service):
        """Test validating audio that is too short."""
        # 100ms of audio = 1600 samples at 16kHz
        samples = 1600
        audio_data = np.zeros(samples, dtype=np.float32)
        audio_buffer = AudioBuffer(
            data=audio_data,
            audio_format=AudioFormat.whisper_format(),
            language=Language("es")
        )
        errors = service.validate_audio_buffer(audio_buffer)
        assert any("too short" in e for e in errors)

    def test_validate_audio_too_long(self, service):
        """Test validating audio that is too long."""
        # 35 seconds of audio at 16kHz = 560000 samples
        samples = 560000
        audio_data = np.zeros(samples, dtype=np.float32)
        audio_buffer = AudioBuffer(
            data=audio_data,
            audio_format=AudioFormat.whisper_format(),
            language=Language("es")
        )
        errors = service.validate_audio_buffer(audio_buffer)
        assert any("too long" in e for e in errors)

    def test_validate_incompatible_format(self, service):
        """Test validating audio with incompatible format."""
        samples = 16000
        audio_data = np.zeros(samples, dtype=np.float32)
        # Create a non-whisper compatible format (wrong sample rate and channels)
        from src.domain.value_objects.audio_format import AudioEncoding
        audio_format = AudioFormat(
            sample_rate=44100,
            channels=2,
            encoding=AudioEncoding.PCM_F32LE
        )
        audio_buffer = AudioBuffer(
            data=audio_data,
            audio_format=audio_format,
            language=Language("es")
        )
        errors = service.validate_audio_buffer(audio_buffer)
        assert any("not compatible with Whisper" in e for e in errors)

    def test_is_valid_language_valid(self, service):
        """Test is_valid_language with valid language."""
        assert service.is_valid_language("es") is True
        assert service.is_valid_language("en") is True
        assert service.is_valid_language("fr") is True

    def test_is_valid_language_invalid(self, service):
        """Test is_valid_language with invalid language."""
        assert service.is_valid_language("xx") is False
        assert service.is_valid_language("invalid") is False

    def test_calculate_expected_processing_time_cpu(self, service):
        """Test calculating expected processing time on CPU."""
        # 1000ms audio should take about 700ms on CPU (0.7 factor)
        expected_time = service.calculate_expected_processing_time(1000, is_cpu=True)
        assert expected_time == 700

    def test_calculate_expected_processing_time_gpu(self, service):
        """Test calculating expected processing time on GPU."""
        # 1000ms audio should take about 100ms on GPU (0.1 factor)
        expected_time = service.calculate_expected_processing_time(1000, is_cpu=False)
        assert expected_time == 100

    def test_should_process_realtime_true(self, service):
        """Test should_process_realtime returns True when faster than realtime."""
        # Processing 1000ms audio in 500ms = realtime
        assert service.should_process_realtime(500, 1000) is True

    def test_should_process_realtime_false(self, service):
        """Test should_process_realtime returns False when slower than realtime."""
        # Processing 1000ms audio in 1500ms = not realtime
        assert service.should_process_realtime(1500, 1000) is False

    def test_should_process_realtime_zero_duration(self, service):
        """Test should_process_realtime handles zero duration."""
        assert service.should_process_realtime(100, 0) is False

    def test_create_empty_transcription(self, service):
        """Test creating an empty transcription."""
        language = Language("es")
        transcription = service.create_empty_transcription(language, 1000)

        assert transcription.text == ""
        assert transcription.language == language
        assert transcription.confidence == 0.0
        assert transcription.audio_duration_ms == 1000
        assert transcription.processing_time_ms == 0

    def test_create_empty_transcription_default_duration(self, service):
        """Test creating an empty transcription with default duration."""
        language = Language("en")
        transcription = service.create_empty_transcription(language)

        assert transcription.audio_duration_ms == 0

    def test_normalize_transcription_text_basic(self, service):
        """Test normalizing basic text."""
        result = service.normalize_transcription_text("hello world")
        assert result == "Hello world"

    def test_normalize_transcription_text_multiple_spaces(self, service):
        """Test normalizing text with multiple spaces."""
        result = service.normalize_transcription_text("hello    world")
        assert result == "Hello world"

    def test_normalize_transcription_text_empty(self, service):
        """Test normalizing empty text."""
        result = service.normalize_transcription_text("")
        assert result == ""

    def test_normalize_transcription_text_already_capitalized(self, service):
        """Test normalizing already capitalized text."""
        result = service.normalize_transcription_text("Hello world")
        assert result == "Hello world"

    def test_normalize_transcription_text_with_whitespace(self, service):
        """Test normalizing text with leading/trailing whitespace."""
        result = service.normalize_transcription_text("  hello world  ")
        assert result == "Hello world"

    def test_normalize_transcription_text_none(self, service):
        """Test normalizing None-like text."""
        result = service.normalize_transcription_text(None)
        assert result == ""
