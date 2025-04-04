import sublime_plugin
import sublime
import subprocess
import os


class InsertFormulaCommand(sublime_plugin.TextCommand):
	def run(self, edit): 
		settings = sublime.load_settings("InsertFormula.sublime-settings")

		command = [os.path.expanduser(settings["math_preview_exe_path"])]

		# Get current selection
		sel    = self.view.sel()
		region = next(iter(sel))
		# If is in a LaTeX scope, we extend selection to that scope
		if region.a == region.b:
			self.view.expand_to_scope("text.tex.latex meta.environment.math")
			
		text   = self.view.substr(region)
		command.extend(["-i", text])

		if "math_font" in settings:
			command.extend(["-m", os.path.expanduser(settings["math_font"])])

		if "sty_file" in settings:
			command.extend(["-y", os.path.expanduser(settings["sty_file"])])


		sublime.set_timeout_async(lambda: self.get_formula(command), 0)


	def get_formula(self, command):
		process = subprocess.run(
			command, 
			stdout = subprocess.PIPE, 
			stderr = subprocess.PIPE, 
		)

		if process.returncode == 0:
			stdout = process.stdout.decode("utf8")
			formula = stdout.strip()
			self.view.run_command("insert_formula_aux", {"formula" : formula})

		else:
			stderr = process.stderr.decode("utf8")
			print("!!! ERROR !!!")
			print(stderr)



class InsertFormulaAuxCommand(sublime_plugin.TextCommand):

	def run(self, edit, formula):
		sel  = self.view.sel()
		region = next(iter(sel))
		self.view.replace(edit, region, formula)


