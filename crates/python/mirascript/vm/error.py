class VmError(Exception):
    """VM 预期的错误"""

    def __init__(self, message, recovered):
        super().__init__(message)
        self.message = message
        self.recovered = recovered

    @staticmethod
    def from_(prefix, error, recovered):
        if prefix and not prefix.endswith(":"):
            prefix += ":"
        vmError = VmError(f"{prefix}{str(error)}", recovered)

        return vmError
