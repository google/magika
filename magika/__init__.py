import logging
from pathlib import Path
import sys

import dotenv
from rich.logging import RichHandler
from rich.console import Console

# make sure that the env is loaded as first thing

def load_env():
    env_dir = Path(__file__).parent
    while True:
        env_path = env_dir / '.env'
        if env_path.is_file():
            break
        if env_dir == Path('/'):
            break
        env_dir = env_dir.parent
    if env_path.is_file():
        dotenv.load_dotenv(env_path, override=True)

load_env()


def get_logger(logger_name='magika'):
    FORMAT = "%(name)s: %(message)s"
    logging.basicConfig(
        level="INFO", format=FORMAT, datefmt="[%x %X]", handlers=[
            RichHandler(console=Console(stderr=True))]
    )
    l = logging.getLogger(logger_name)
    return l


# disable annoying logging from google library: https://github.com/googleapis/google-api-python-client/issues/299
logging.getLogger('googleapiclient.discovery_cache').setLevel(logging.ERROR)
