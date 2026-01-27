"""Tests para Value Objects del dominio."""

import pytest
from src.domain.value_objects import Language, AudioFormat, AudioEncoding


class TestLanguage:
    """Tests para el Value Object Language."""

    def test_create_valid_language(self):
        """Test crear idioma válido."""
        lang = Language("es")
        assert lang.code == "es"
        assert lang.name == "Spanish"

    def test_create_language_normalizes_to_lowercase(self):
        """Test que el código se normaliza a minúsculas."""
        lang = Language("ES")
        assert lang.code == "es"

    def test_create_invalid_language_raises_error(self):
        """Test que idioma inválido lanza error."""
        with pytest.raises(ValueError, match="Unsupported language"):
            Language("xx")

    def test_create_empty_language_raises_error(self):
        """Test que idioma vacío lanza error."""
        with pytest.raises(ValueError, match="cannot be empty"):
            Language("")

    def test_language_equality(self):
        """Test igualdad entre idiomas."""
        lang1 = Language("es")
        lang2 = Language("es")
        lang3 = Language("en")

        assert lang1 == lang2
        assert lang1 != lang3
        assert lang1 == "es"

    def test_factory_methods(self):
        """Test métodos factory."""
        assert Language.spanish().code == "es"
        assert Language.english().code == "en"
        assert Language.portuguese().code == "pt"

    def test_language_is_hashable(self):
        """Test que Language es hashable (puede usarse en sets/dicts)."""
        lang1 = Language("es")
        lang2 = Language("es")
        lang_set = {lang1, lang2}
        assert len(lang_set) == 1


class TestAudioFormat:
    """Tests para el Value Object AudioFormat."""

    def test_create_valid_audio_format(self):
        """Test crear formato de audio válido."""
        fmt = AudioFormat(
            sample_rate=16000,
            channels=1,
            encoding=AudioEncoding.PCM_F32LE
        )
        assert fmt.sample_rate == 16000
        assert fmt.channels == 1
        assert fmt.encoding == AudioEncoding.PCM_F32LE

    def test_create_invalid_sample_rate_raises_error(self):
        """Test que sample rate inválido lanza error."""
        with pytest.raises(ValueError, match="must be positive"):
            AudioFormat(sample_rate=0, channels=1, encoding=AudioEncoding.WAV)

        with pytest.raises(ValueError, match="too high"):
            AudioFormat(sample_rate=200000, channels=1, encoding=AudioEncoding.WAV)

    def test_create_invalid_channels_raises_error(self):
        """Test que canales inválidos lanzan error."""
        with pytest.raises(ValueError, match="must be positive"):
            AudioFormat(sample_rate=16000, channels=0, encoding=AudioEncoding.WAV)

        with pytest.raises(ValueError, match="Too many channels"):
            AudioFormat(sample_rate=16000, channels=10, encoding=AudioEncoding.WAV)

    def test_whisper_format_factory(self):
        """Test factory para formato Whisper."""
        fmt = AudioFormat.whisper_format()
        assert fmt.sample_rate == 16000
        assert fmt.channels == 1
        assert fmt.encoding == AudioEncoding.PCM_F32LE

    def test_is_whisper_compatible(self):
        """Test verificación de compatibilidad con Whisper."""
        compatible = AudioFormat.whisper_format()
        assert compatible.is_whisper_compatible()

        incompatible = AudioFormat(
            sample_rate=44100,
            channels=2,
            encoding=AudioEncoding.WAV
        )
        assert not incompatible.is_whisper_compatible()

    def test_duration_from_samples(self):
        """Test cálculo de duración desde samples."""
        fmt = AudioFormat.whisper_format()
        # 16000 samples = 1 segundo a 16kHz
        assert fmt.duration_from_samples(16000) == 1.0
        assert fmt.duration_from_samples(8000) == 0.5

    def test_samples_from_duration(self):
        """Test cálculo de samples desde duración."""
        fmt = AudioFormat.whisper_format()
        assert fmt.samples_from_duration(1.0) == 16000
        assert fmt.samples_from_duration(0.5) == 8000

    def test_bytes_per_second(self):
        """Test cálculo de bytes por segundo."""
        fmt = AudioFormat.whisper_format()
        # PCM_F32LE = 4 bytes por sample, 16000 Hz, 1 canal
        assert fmt.bytes_per_second() == 64000
