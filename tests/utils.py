import random
import string

def get_random_ascii_bytes(size):
    return bytes([random.choice(bytes(string.ascii_letters.encode('ascii'))) for _ in range(size)])