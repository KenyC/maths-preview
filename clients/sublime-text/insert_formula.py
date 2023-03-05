import sublime_plugin
import sublime
import subprocess
import os


class InsertFormulaCommand(sublime_plugin.TextCommand):
	def run(self, edit): 
		settings = sublime.load_settings("InsertFormula.sublime-settings")
		# print(settings["math_preview_exe_path"])

		env = os.environ.copy()
		if "math_font" in settings:
			env["MATH_FONT"] = settings["math_font"]

		exe = os.path.expanduser(settings["math_preview_exe_path"])

		sublime.set_timeout_async(lambda: self.get_formula(exe, env), 0)


	def get_formula(self, exe, env):
		process = subprocess.run(exe, stdout = subprocess.PIPE, stderr = subprocess.PIPE, shell = True, env = env)

		if process.returncode == 0:
			stdout = process.stdout.decode("utf8")
			formula = stdout.strip()
			self.view.run_command("insert_formula_aux", {"formula" : formula})

		else:
			print("!!! ERROR !!!")
			print(process.stderr.decode("utf8"))



class InsertFormulaAuxCommand(sublime_plugin.TextCommand):

	def run(self, edit, formula):
		sel  = self.view.sel()
		region = next(iter(sel))
		self.view.replace(edit, region, formula)


