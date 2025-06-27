class Config:
    """
    MiraScript 编译配置

    Attributes:
        input_mode (str): 输入模式，支持 'script' 和 'template'
    """

    def __init__(self, **data):
        """
        初始化编译配置

        Args:
            **input_mode (str): 输入模式，支持 'script' 和 'template'
        """
        ...

def compile(script: str, config: Config) -> tuple[bytes | None, list[int]]:
    """
    编译 MiraScript 代码，生成字节码

    Args:
        script (str): 要编译的 MiraScript 代码
        config (Config): 编译配置

    Returns:
        tuple[bytes | None, list[int]]: 编译后的字节码和诊断信息
    """
    ...
