"""Tests for application use cases."""

import pytest
from unittest.mock import MagicMock, AsyncMock
import time

from src.application.use_cases.health_check import HealthCheckUseCase, ServiceInfo
from src.application.dtos import HealthResponse, ModelsResponse


class TestHealthCheckUseCase:
    """Tests for HealthCheckUseCase."""

    @pytest.fixture
    def mock_speech_recognition(self):
        """Create mock speech recognition port."""
        mock = MagicMock()
        mock.is_available = AsyncMock(return_value=True)
        mock.get_model_info.return_value = {
            "name": "large-v3-turbo",
            "device": "cpu",
            "compute_type": "int8"
        }
        mock.get_supported_languages.return_value = ["es", "en", "fr"]
        return mock

    @pytest.fixture
    def service_info(self):
        """Create service info."""
        return ServiceInfo(version="1.0.0", start_time=time.time() - 100)

    @pytest.fixture
    def use_case(self, mock_speech_recognition, service_info):
        """Create use case with mocks."""
        return HealthCheckUseCase(mock_speech_recognition, service_info)

    @pytest.mark.asyncio
    async def test_execute_healthy(self, use_case):
        """Test health check when service is healthy."""
        response = await use_case.execute()

        assert isinstance(response, HealthResponse)
        assert response.status == "healthy"
        assert response.version == "1.0.0"
        assert response.model == "large-v3-turbo"
        assert response.device == "cpu"
        assert response.uptime_seconds >= 100

    @pytest.mark.asyncio
    async def test_execute_unhealthy(self, mock_speech_recognition, service_info):
        """Test health check when service is unhealthy."""
        mock_speech_recognition.is_available = AsyncMock(return_value=False)
        use_case = HealthCheckUseCase(mock_speech_recognition, service_info)

        response = await use_case.execute()

        assert response.status == "unhealthy"

    def test_get_models(self, use_case):
        """Test get models."""
        response = use_case.get_models()

        assert isinstance(response, ModelsResponse)
        assert len(response.models) == 1
        assert response.models[0]["name"] == "large-v3-turbo"
        assert response.models[0]["device"] == "cpu"
        assert response.models[0]["compute_type"] == "int8"
        assert response.models[0]["languages"] == ["es", "en", "fr"]
        assert response.current_model == "large-v3-turbo"

    def test_get_models_with_unknown_model(self, mock_speech_recognition, service_info):
        """Test get models when model info is incomplete."""
        mock_speech_recognition.get_model_info.return_value = {}
        use_case = HealthCheckUseCase(mock_speech_recognition, service_info)

        response = use_case.get_models()

        assert response.current_model == "unknown"
        assert response.models[0]["name"] == "unknown"
        assert response.models[0]["device"] == "cpu"


class TestServiceInfo:
    """Tests for ServiceInfo dataclass."""

    def test_default_values(self):
        """Test default values."""
        info = ServiceInfo()
        assert info.version == "1.0.0"
        assert info.start_time == 0.0

    def test_custom_values(self):
        """Test custom values."""
        info = ServiceInfo(version="2.0.0", start_time=1000.0)
        assert info.version == "2.0.0"
        assert info.start_time == 1000.0
