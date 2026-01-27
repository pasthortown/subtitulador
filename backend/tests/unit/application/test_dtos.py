"""Tests for application DTOs."""

import base64
import pytest
from pydantic import ValidationError

from src.application.dtos.transcription_request import (
    TranscriptionRequest,
    TranscriptionStreamRequest
)
from src.application.dtos.transcription_response import (
    TranscriptionData,
    ErrorData,
    TranscriptionResponse,
    HealthResponse,
    ModelsResponse
)


class TestTranscriptionRequest:
    """Tests for TranscriptionRequest DTO."""

    def test_valid_request(self):
        """Test creating a valid request."""
        # Create valid base64 audio (at least 100 bytes)
        audio_data = b'\x00' * 200
        audio_b64 = base64.b64encode(audio_data).decode()

        request = TranscriptionRequest(audio=audio_b64)
        assert request.language == "es"
        assert request.sample_rate == 16000

    def test_custom_language(self):
        """Test request with custom language."""
        audio_data = b'\x00' * 200
        audio_b64 = base64.b64encode(audio_data).decode()

        request = TranscriptionRequest(audio=audio_b64, language="en")
        assert request.language == "en"

    def test_language_normalized_to_lowercase(self):
        """Test that language is normalized to lowercase."""
        audio_data = b'\x00' * 200
        audio_b64 = base64.b64encode(audio_data).decode()

        request = TranscriptionRequest(audio=audio_b64, language="EN")
        assert request.language == "en"

    def test_invalid_language_raises_error(self):
        """Test that invalid language raises error."""
        audio_data = b'\x00' * 200
        audio_b64 = base64.b64encode(audio_data).decode()

        with pytest.raises(ValidationError):
            TranscriptionRequest(audio=audio_b64, language="xx")

    def test_invalid_base64_raises_error(self):
        """Test that invalid base64 raises error."""
        with pytest.raises(ValidationError):
            TranscriptionRequest(audio="not-valid-base64!!!")

    def test_audio_too_small_raises_error(self):
        """Test that audio data too small raises error."""
        audio_data = b'\x00' * 10
        audio_b64 = base64.b64encode(audio_data).decode()

        with pytest.raises(ValidationError):
            TranscriptionRequest(audio=audio_b64)

    def test_sample_rate_validation(self):
        """Test sample rate validation."""
        audio_data = b'\x00' * 200
        audio_b64 = base64.b64encode(audio_data).decode()

        # Valid sample rates
        request = TranscriptionRequest(audio=audio_b64, sample_rate=8000)
        assert request.sample_rate == 8000

        request = TranscriptionRequest(audio=audio_b64, sample_rate=48000)
        assert request.sample_rate == 48000

        # Invalid sample rate
        with pytest.raises(ValidationError):
            TranscriptionRequest(audio=audio_b64, sample_rate=1000)


class TestTranscriptionStreamRequest:
    """Tests for TranscriptionStreamRequest DTO."""

    def test_default_values(self):
        """Test default values."""
        request = TranscriptionStreamRequest()
        assert request.language == "es"
        assert request.sample_rate == 16000
        assert request.chunk_duration_ms == 100

    def test_custom_values(self):
        """Test custom values."""
        request = TranscriptionStreamRequest(
            language="en",
            sample_rate=44100,
            chunk_duration_ms=500
        )
        assert request.language == "en"
        assert request.sample_rate == 44100
        assert request.chunk_duration_ms == 500

    def test_invalid_language_raises_error(self):
        """Test invalid language raises error."""
        with pytest.raises(ValidationError):
            TranscriptionStreamRequest(language="invalid")

    def test_chunk_duration_validation(self):
        """Test chunk duration validation."""
        # Valid range
        request = TranscriptionStreamRequest(chunk_duration_ms=50)
        assert request.chunk_duration_ms == 50

        request = TranscriptionStreamRequest(chunk_duration_ms=1000)
        assert request.chunk_duration_ms == 1000

        # Invalid range
        with pytest.raises(ValidationError):
            TranscriptionStreamRequest(chunk_duration_ms=10)

        with pytest.raises(ValidationError):
            TranscriptionStreamRequest(chunk_duration_ms=2000)


class TestTranscriptionData:
    """Tests for TranscriptionData DTO."""

    def test_valid_data(self):
        """Test creating valid transcription data."""
        data = TranscriptionData(
            text="Hello world",
            language="en",
            confidence=0.95,
            duration_ms=1000,
            processing_time_ms=50
        )
        assert data.text == "Hello world"
        assert data.language == "en"
        assert data.confidence == 0.95

    def test_confidence_validation(self):
        """Test confidence validation."""
        with pytest.raises(ValidationError):
            TranscriptionData(
                text="test",
                language="es",
                confidence=1.5,  # Invalid: > 1.0
                duration_ms=100,
                processing_time_ms=10
            )


class TestErrorData:
    """Tests for ErrorData DTO."""

    def test_error_without_details(self):
        """Test error without details."""
        error = ErrorData(code="TEST_ERROR", message="Test message")
        assert error.code == "TEST_ERROR"
        assert error.message == "Test message"
        assert error.details is None

    def test_error_with_details(self):
        """Test error with details."""
        error = ErrorData(
            code="TEST_ERROR",
            message="Test message",
            details={"key": "value"}
        )
        assert error.details == {"key": "value"}


class TestTranscriptionResponse:
    """Tests for TranscriptionResponse DTO."""

    def test_ok_factory(self):
        """Test ok factory method."""
        response = TranscriptionResponse.ok(
            text="Hello",
            language="en",
            confidence=0.9,
            duration_ms=500,
            processing_time_ms=25
        )
        assert response.success is True
        assert response.data is not None
        assert response.data.text == "Hello"
        assert response.error is None

    def test_failure_factory(self):
        """Test failure factory method."""
        response = TranscriptionResponse.failure(
            code="ERROR",
            message="Something went wrong"
        )
        assert response.success is False
        assert response.data is None
        assert response.error is not None
        assert response.error.code == "ERROR"

    def test_failure_with_details(self):
        """Test failure factory with details."""
        response = TranscriptionResponse.failure(
            code="ERROR",
            message="Error",
            details={"info": "test"}
        )
        assert response.error.details == {"info": "test"}


class TestHealthResponse:
    """Tests for HealthResponse DTO."""

    def test_health_response(self):
        """Test creating health response."""
        response = HealthResponse(
            status="healthy",
            version="1.0.0",
            model="whisper-large",
            device="cpu",
            uptime_seconds=3600.0
        )
        assert response.status == "healthy"
        assert response.version == "1.0.0"
        assert response.model == "whisper-large"


class TestModelsResponse:
    """Tests for ModelsResponse DTO."""

    def test_models_response(self):
        """Test creating models response."""
        response = ModelsResponse(
            models=[{"name": "large-v3-turbo", "device": "cpu"}],
            current_model="large-v3-turbo"
        )
        assert len(response.models) == 1
        assert response.current_model == "large-v3-turbo"
