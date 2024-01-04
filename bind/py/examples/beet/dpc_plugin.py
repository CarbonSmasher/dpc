from typing import ClassVar
import datapack_compiler as dpc
from beet import Context, TextFile

def beet_default(ctx: Context):
  ctx.data.extend_namespace.append(IRFile)

class IRFile(TextFile):
  scope: ClassVar[tuple[str, ...]] = ("ir",)
  extension: ClassVar[str] = ".dpc"
