# scripts/check_pyproject_consistency.py
import toml
import re

def check_consistency():
    # Load pyproject.toml
    data = toml.load('python/pyproject.toml')
    
    # Extract supported Python versions from classifiers
    classifiers = data['project']['classifiers']
    python_versions = [c.split('::')[-1].strip() for c in classifiers if 'Programming Language :: Python :: 3.' in c]
    
    # Extract platforms
    platforms = [c.split('::')[-1].strip() for c in classifiers if 'Operating System ::' in c]
    
    # Read README.md and check for consistency
    with open('README.md', 'r') as f:
        readme = f.read()
    
    # Check if versions/platforms are mentioned
    for version in python_versions:
        if version not in readme:
            print(f"Warning: Python {version} not found in README.md")
    
    for platform in platforms:
        if platform.lower() not in readme.lower():
            print(f"Warning: Platform {platform} not found in README.md")
    
    print("Consistency check complete.")

if __name__ == "__main__":
    check_consistency()