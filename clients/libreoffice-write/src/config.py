# coding: utf-8

from __future__ import unicode_literals

import uno
import unohelper
import ast
import traceback
import webbrowser
from sys import getfilesystemencoding, version_info
from threading import Thread
from subprocess import call as sub_call

try:
	import pythonscript
except ImportError:
	import pythonloader
	pythonscript = None
	for url, module in pythonloader.g_loadedComponents.iteritems():
		if url.endswith("script-provider-for-python/pythonscript.py"):
			pythonscript = module
	if pythonscript is None:
		raise Exception("Impossible de trouver le module pythonscript.")

from com.sun.star.beans import PropertyValue
from com.sun.star.lang import Locale, IllegalArgumentException
from com.sun.star.task import XJobExecutor
from com.sun.star.awt import (XActionListener, XMouseListener,  XKeyListener,
							  XContainerWindowEventHandler, Rectangle, Selection)
from com.sun.star.awt.KeyModifier import MOD1
from com.sun.star.awt.PosSize import POS, SIZE, POSSIZE
from com.sun.star.awt.tree import XTreeExpansionListener
from com.sun.star.view import XSelectionChangeListener
from com.sun.star.uno import Exception as UNOException
from com.sun.star.awt.MessageBoxType import MESSAGEBOX, ERRORBOX, WARNINGBOX
from com.sun.star.awt.MessageBoxResults import YES
from com.sun.star.ui.dialogs.TemplateDescription import FILESAVE_AUTOEXTENSION, FILEOPEN_SIMPLE



RR = None
EXTID = 'org.kenyc.intsertfrommathspreview'



class ResourceResolver(object):
	'''Resource Resolver for localized strings'''
	def __init__(self, ctx):
		self.ctx = ctx
		self.smgr = self.ctx.getServiceManager()
		self.locale = self._get_env_locale()
		self.srwl = self._get_resource_resolver()
		self.version = self._get_ext_ver()

	# def _get_ext_path(self):
	#     '''Get addon installation path'''
	#     pip = self.ctx.getByName(
	#         "/singletons/com.sun.star.deployment.PackageInformationProvider")
	#     extpath = pip.getPackageLocation(EXTID)
	#     if extpath[-1] != "/":
	#         extpath += "/"
	#     return extpath

	def _get_ext_ver(self):
		'''Get addon version number'''
		pip = self.ctx.getByName(
			   "/singletons/com.sun.star.deployment.PackageInformationProvider")
		extensions = pip.getExtensionList()
		for ext in extensions:
			if EXTID in ext:
				return ext[1]
		return ''

	def _get_env_locale(self):
		'''Get interface locale'''
		ps = self.smgr.createInstanceWithContext(
			"com.sun.star.util.PathSubstitution", self.ctx)
		vlang = ps.getSubstituteVariableValue("vlang")
		alang = vlang.split("-") + 2*[""]
		locale = Locale(*alang[:3])
		return locale

	def _get_resource_resolver(self):
		# url = self._get_ext_path() + "python"
		url = "vnd.sun.star.extension://{}/src".format(EXTID)
		handler = self.smgr.createInstanceWithContext(
			"com.sun.star.task.InteractionHandler", self.ctx)
		srwl = self.smgr.createInstanceWithArgumentsAndContext(
			"com.sun.star.resource.StringResourceWithLocation",
			(url, False, self.locale, "strings", "", handler), self.ctx)
		return srwl

	def resolvestring(self, id):
		return self.srwl.resolveString(id)


def loadResourceResolver(ctx):
	print("!!!!!!!!!!!!!!!!!!!")
	global RR
	if not RR:
		RR = ResourceResolver(ctx)


def createUnoService(service, ctx=None, args=None):
    '''
    Instanciate a Uno service.

    @service: name of the service to be instanciated.
    @ctx: the context if required.
    @args: the arguments when needed.
    '''
    if not ctx:
        ctx = uno.getComponentContext()
    smgr = ctx.getServiceManager()
    if ctx and args:
        return smgr.createInstanceWithArgumentsAndContext(service, args, ctx)
    elif args:
        return smgr.createInstanceWithArguments(service, args)
    elif ctx:
        return smgr.createInstanceWithContext(service, ctx)
    else:
        return smgr.createInstance(service)



def getConfigurationAccess(nodevalue, updatable=False):
	'''
	Access configuration value.

	@nodevalue: the configuration key node as a string.
	@updatable: set True when accessor needs to modify the key value.
	'''
	cp = createUnoService("com.sun.star.configuration.ConfigurationProvider")
	node = PropertyValue("nodepath", 0, nodevalue, 0)
	print("fezfez")
	if updatable:
		to_return = cp.createInstanceWithArguments("com.sun.star.configuration.ConfigurationUpdateAccess", (node,))
		print("fez1")
		return to_return
	else:
		to_return = cp.createInstanceWithArguments("com.sun.star.configuration.ConfigurationAccess", (node,))
		print("fez")
		return to_return


# -----------------------------------------------------------
# EDITOR KICKER
# -----------------------------------------------------------

# uno implementation
g_ImplementationHelper = unohelper.ImplementationHelper()

ImplementationName = "InsertFromMathsPreview.EditorKickerOptionsPage"


class ButtonListener(unohelper.Base, XActionListener):
	def __init__(self, cast):
		self.cast = cast

	def disposing(self, ev):
		pass

	def actionPerformed(self, ev):
		cmd = str(ev.ActionCommand)
		if cmd == "ChooseEditor":
			ret = self.cast.chooseFile()
			if ret:
				path = uno.fileUrlToSystemPath(ret)
				ev.Source.getContext().getControl("tf_Editor").setText(path)

# main class
class OptionsDialogHandler(unohelper.Base, XContainerWindowEventHandler):
	def __init__(self, ctx):
		self.ctx = ctx
		loadResourceResolver(self.ctx)
		self.CfgNode = "/InsertFromMathsPreview.Settings/EditorKicker"

	# XContainerWindowEventHandler
	def callHandlerMethod(self, window, eventObject, method):
		if method == "external_event":
			try:
				self._handleExternalEvent(window, eventObject)
			except Exception as e:
				print(e)
			return True

	# XContainerWindowEventHandler
	def getSupportedMethodNames(self):
		return ("external_event",)

	def _handleExternalEvent(self, window, evName):
		if evName == "ok":
			self._saveData(window)
		elif evName == "back":
			self._loadData(window, "back")
		elif evName == "initialize":
			self._loadData(window, "initialize")
		return True

	def _saveData(self, window):
		name = window.getModel().Name
		if name != "InsertFromMathsPreview_AllOptions":
			return
		editor = window.getControl("tf_Editor")
		options = window.getControl("tf_Options")
		header = window.getControl("tf_Header")
		settings = {"names": ("EditorPath", "EditorArgs", "DefaultHeader"), "values": (editor.Text, options.Text, header.Text)}
		print("Writing", settings)
		self._configwriter(settings)

	def _loadData(self, window, evName):
		name = window.getModel().Name
		if name != "InsertFromMathsPreview_AllOptions":
			return
		if evName == "initialize":
			listener = ButtonListener(self)
			btn_Choose = window.getControl("btn_Choose")
			btn_Choose.ActionCommand = "ChooseEditor"
			btn_Choose.addActionListener(listener)
			for control in window.Controls:
				if not control.supportsService("com.sun.star.awt.UnoControlEdit"):
					model = control.Model
					model.Label = RR.resolvestring(model.Label)
		settings = self._configreader()
		print(settings)
		for k, v in settings.items():
			if v is None:
				settings[k] = ""
		print(settings)

		if settings:
			tf_Editor = window.getControl("tf_Editor")
			tf_Options = window.getControl("tf_Options")
			tf_Header = window.getControl("tf_Header")
			tf_Editor.setText(settings["EditorPath"])
			tf_Options.setText(settings["EditorArgs"])
			tf_Header.setText(settings["DefaultHeader"])
		print("rre")
		return

	def _configreader(self):
		settings = {}
		try:
			reader = getConfigurationAccess(self.CfgNode)
			names = reader.ElementNames
			values = reader.getPropertyValues(names)
			settings = {k: v for k, v in zip(names, values)}
		except Exception as e:
			raise e
		return settings

	def _configwriter(self, settings):
		try:
			writer = getConfigurationAccess(self.CfgNode, True)
			writer.setPropertyValues(settings["names"], settings["values"])
			writer.commitChanges()
		except Exception as e:
			raise e

	def chooseFile(self):
		# ret = self._getFileUrl()
		# return ret
		return ""

	# def _getFileUrl(self):
	# 	url = FileOpenDialog(self.ctx,
	# 						 template=FILEOPEN_SIMPLE,
	# 						 filters=((RR.resolvestring('msg07'), '*.*'),
	# 								  (RR.resolvestring('ek10'), '*.exe;*.bin;*.sh'))).execute()
	# 	return url or False


g_ImplementationHelper.addImplementation(
	OptionsDialogHandler, ImplementationName, (ImplementationName,),)

