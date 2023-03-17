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
from utils import *



RR = None
EXTID = 'org.kenyc.intsertfrommathspreview'





#######################################
# CONFIGURATION DIALOG 
#######################################

# uno implementation
g_ImplementationHelper = unohelper.ImplementationHelper()

ImplementationName = "InsertFromMathsPreview.AllOptionsPage"


class ButtonListener(unohelper.Base, XActionListener):
	def __init__(self, cast, closure):
		self.cast    = cast
		self.closure = closure

	def disposing(self, ev):
		pass

	def actionPerformed(self, ev):
		self.closure(self.cast, ev)


class OptionsDialogHandler(unohelper.Base, XContainerWindowEventHandler):
	def __init__(self, ctx):
		self.ctx = ctx
		loadResourceResolver(self.ctx)
		self.CfgNode = "/InsertFromMathsPreview.Settings/AllOptions"

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
		editor  = window.getControl("tf_MathsPreviewPath")
		options = window.getControl("tf_MathsFont")
		settings = {"names": ("MathsPreviewPath", "MathsFont"), "values": (editor.Text, options.Text)}
		self._configwriter(settings)

	def _loadData(self, window, evName):
		name = window.getModel().Name
		if name != "InsertFromMathsPreview_AllOptions":
			return
		if evName == "initialize":
			self.setup_buttons(window)
			for control in window.Controls:
				if not control.supportsService("com.sun.star.awt.UnoControlEdit"):
					model = control.Model
					model.Label = RR.resolvestring(model.Label)
		settings = self._configreader()
		for k, v in settings.items():
			if v is None:
				settings[k] = ""

		if settings:
			tf_MathsPreviewPath = window.getControl("tf_MathsPreviewPath")
			tf_MathsFont = window.getControl("tf_MathsFont")
			tf_MathsPreviewPath.setText(settings["MathsPreviewPath"])
			tf_MathsFont.setText(settings["MathsFont"])
		return

	def setup_buttons(self, window):
		exe_filters = (
			(RR.resolvestring('msg07'), '*.*'),
			(RR.resolvestring('ek10'), '*.exe;*.bin;*.sh')
		)
		listener = ButtonListener(self, lambda obj, ev: 
			obj.chooseFile(ev, exe_filters, "tf_MathsPreviewPath") if str(ev.ActionCommand) == "ChooseEditor" else None
		)
		btn_PickExe = window.getControl("btn_PickExe")
		btn_PickExe.ActionCommand = "ChooseEditor"
		btn_PickExe.addActionListener(listener)

		otf_filters = (
			(RR.resolvestring('msg07'), '*.*'),
			(RR.resolvestring('ek11'),  '*.otf')
		)
		listener = ButtonListener(self, lambda obj, ev: 
			obj.chooseFile(ev, otf_filters, "tf_MathsFont") if str(ev.ActionCommand) == "ChooseEditor" else None
		)
		btn_PickExe = window.getControl("btn_PickFont")
		btn_PickExe.ActionCommand = "ChooseEditor"
		btn_PickExe.addActionListener(listener)


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

	def chooseFile(self, ev, filters, field_name):
		url = FileOpenDialog(
			self.ctx,
			template=FILEOPEN_SIMPLE,
			filters=filters
		).execute()

		if url:
			path = uno.fileUrlToSystemPath(url)
			ev.Source.getContext().getControl(field_name).setText(path)

g_ImplementationHelper.addImplementation(
	OptionsDialogHandler, ImplementationName, (ImplementationName,),)


#######################################
# VARIOUS UTILS
#######################################




class ResourceResolver(object):
	'''Resource Resolver for localized strings'''
	def __init__(self, ctx):
		self.ctx = ctx
		self.smgr = self.ctx.getServiceManager()
		self.locale = self._get_env_locale()
		self.srwl = self._get_resource_resolver()
		self.version = self._get_ext_ver()


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
	global RR
	if not RR:
		RR = ResourceResolver(ctx)




class DialogBase(object):
    """ Base class for dialog. """
    def __init__(self, ctx):
        self.ctx = ctx
        self.smgr = ctx.getServiceManager()

    def create(self, name, arguments=None):
        """ Create service instance. """
        if arguments:
            return self.smgr.createInstanceWithArgumentsAndContext(
                name, arguments, self.ctx)
        else:
            return self.smgr.createInstanceWithContext(
                name, self.ctx)


class FileOpenDialog(DialogBase):
    """ To get file url to open. """
    def __init__(self, ctx, **kwds):
        DialogBase.__init__(self, ctx)
        self.args = kwds
        AvailableServiceNames = self.ctx.getServiceManager().getAvailableServiceNames()
        if "com.sun.star.ui.dialogs.SystemFilePicker" in AvailableServiceNames:
            self.filepickerservice = "com.sun.star.ui.dialogs.SystemFilePicker"
        elif "com.sun.star.ui.dialogs.GtkFilePicker" in AvailableServiceNames:
            self.filepickerservice = "com.sun.star.ui.dialogs.GtkFilePicker"
        else:
            self.filepickerservice = "com.sun.star.ui.dialogs.FilePicker"

    def execute(self):
        fp = self.create(self.filepickerservice)
        args = self.args
        if "template" in args:
            fp.initialize((args["template"],))
        if "title" in args:
            fp.setTitle(args["title"])
        if "default" in args:
            default = args["default"]
            fp.setDefaultName(self._substitute_variables(default))
        if "directory" in args:
            fp.setDisplayDirectory(args["directory"])
        if "filters" in args:
            for title, filter in args["filters"]:
                fp.appendFilter(title, filter)
        result = None
        if fp.execute():
            result = fp.getFiles()[0]
        return result

    def _substitute_variables(self, uri):
        return self.create("com.sun.star.util.PathSubstitution").\
            substituteVariables(uri, True)

