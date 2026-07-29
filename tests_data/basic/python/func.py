def edge_function(param):
    if param is None:
        return "No value provided"
    elif isinstance(param, int) and param < 0:
        return "Negative value"
    elif isinstance(param, str) and not param.strip():
        return "Empty string"
    return f"Valid input: {param}"

# Test cases
print(edge_function(None))
print(edge_function(-5))
print(edge_function("   "))
print(edge_function("Hello"))