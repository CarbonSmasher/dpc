from typing import List
import mecha
from beet import Context
from mecha import rule, Visitor, AstCommand

def beet_default(ctx: Context):
	mc = ctx.inject(mecha.Mecha)
	mc.serialize.reset()
	mc.serialize.extend(Codegen())

class Codegen(Visitor):
	def __init__(self):
		pass

	@rule(AstCommand)
	def process(self, node: AstCommand, result: List[str]):
		print(node.identifier)
