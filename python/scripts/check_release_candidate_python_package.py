import sys
import subprocess
import magika
import re


def check_python_module_version():
    """Check the version of the Python magika package."""
    errors = []
    module_version = getattr(magika, "__version__", None)
    try:
        instance_version = magika.Magika().get_version()
    except Exception as e:
        instance_version = None
        errors.append(f"ERROR: Failed to get instance version: {e}")

    print(f"[DEBUG] Python Package Version: {module_version}")
    print(f"[DEBUG] Python Magika Instance Version: {instance_version}")

    if module_version is None:
        errors.append("ERROR: Could not retrieve Python package version.")

    # Ensure version is non-dev for Python package
    if module_version and "-dev" in module_version:
        errors.append(
            f"ERROR: Python package version {module_version} contains '-dev'."
        )

    return module_version, instance_version, errors


def check_rust_cli_version():
    """Check the version of the Rust CLI magika, ensuring it's non-dev."""
    errors = []
    cli_version = None
    try:
        result = subprocess.run(["magika", "--version"], capture_output=True, text=True)
        # Assume output like: 'magika 0.1.1 standard_v3_2'
        parts = result.stdout.strip().split()
        if len(parts) >= 2:
            cli_version = parts[1]
            print(f"[DEBUG] CLI Magika Version: {cli_version}")
            if "-dev" in cli_version:
                errors.append(f"ERROR: Rust CLI version {cli_version} contains '-dev'.")
        else:
            errors.append("ERROR: Could not parse CLI version output.")
    except Exception as e:
        errors.append(f"ERROR: Could not retrieve CLI version: {e}")

    return cli_version, errors


def validate_release_tag(extracted_version, release_tag):
    """Ensure extracted Python module version matches release tag."""
    errors = []
    # Handle python-v prefix in release tag if present
    normalized_release_tag = re.sub(r"^python-v", "", release_tag)
    if extracted_version and extracted_version != normalized_release_tag:
        errors.append(
            f"ERROR: Extracted version {extracted_version} does not match release tag {release_tag}."
        )
    return errors


def collect_and_report_errors(errors):
    """Print errors and exit if there are any mismatches."""
    if errors:
        for error in errors:
            print(error)
        sys.exit(1)


def main(expected_version, use_python_client=False):
    errors = []
    # Check Python package versions
    module_version, instance_version, py_errors = check_python_module_version()
    errors.extend(py_errors)

    # Check CLI version if not using python-client fallback
    if not use_python_client:
        cli_version, cli_errors = check_rust_cli_version()
        errors.extend(cli_errors)
        print(
            f"[INFO] CLI version is {cli_version} (Not validated against release tag)."
        )
    else:
        cli_version = instance_version  # Fallback; not validated

    # Validate that the Python module versions match the expected release tag
    errors.extend(validate_release_tag(module_version, expected_version))
    errors.extend(validate_release_tag(instance_version, expected_version))
    # Note: CLI version is not validated against the release tag

    collect_and_report_errors(errors)
    print("✅ All versions match!")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(
            "Usage: python check_release_candidate_python_package.py <expected_version> [--python-client]"
        )
        sys.exit(1)
    expected_version = sys.argv[1]
    use_python_client = "--python-client" in sys.argv
    main(expected_version, use_python_client)
