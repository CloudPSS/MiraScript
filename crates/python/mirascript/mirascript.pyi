# from .diagnostics import DiagnosticLevel

class ConfigData(object):
    """
    MiraScript 编译配置数据

    Attributes:
        input_mode (str): 输入模式，支持 'script' 和 'template'
    """

    input_mode: str

class Config:
    """
    MiraScript 编译配置

    Attributes:
        input_mode (str): 输入模式，支持 'script' 和 'template'
    """

    def __init__(self, **data) -> None:
        """
        初始化编译配置

        Args:
            **input_mode (str): 输入模式，支持 'script' 和 'template'
        """
        ...

def compile(script: str, config: Config):
    """
    编译 MiraScript 代码，生成字节码

    Args:
        script (str): 要编译的 MiraScript 代码
        config (Config): 编译配置

    Returns:
        tuple[bytes | None, list[int]]: 编译后的字节码和诊断信息
    """
    ...

def get_diagnostic_message(code: int):
    """
    获取诊断信息的消息

    Args:
        code (int): 诊断代码

    Returns:
        tuple[Literal["Error", "Warning", "Info", "Hint", "Reference", "Unknown"], str, str]: 级别字符串、诊断消息名称和诊断消息描述
    """
    ...

def op_codes():
    """
    获取 MiraScript 的所有操作码

    Returns:
        dict[str, int]: 操作码名称和对应的整数值
    """
    ...
