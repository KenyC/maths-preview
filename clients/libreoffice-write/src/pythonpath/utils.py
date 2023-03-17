import uno
import unohelper
import ast
import traceback
import webbrowser
from sys import getfilesystemencoding, version_info
from threading import Thread
from subprocess import call as sub_call


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
	if updatable:
		to_return = cp.createInstanceWithArguments("com.sun.star.configuration.ConfigurationUpdateAccess", (node,))
		return to_return
	else:
		to_return = cp.createInstanceWithArguments("com.sun.star.configuration.ConfigurationAccess", (node,))
		return to_return
