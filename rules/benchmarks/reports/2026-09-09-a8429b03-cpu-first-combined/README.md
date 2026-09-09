# Superseded scheduling candidate

This candidate kept the CPU worker pool active after GPU preparation completed.
Its [measured comparison](comparison.md) is retained, including the bulk
regression. Revision `65505ce3` instead lets those CPU workers finish their
claimed batch and retires them when GPU work starts.
