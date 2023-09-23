# coding: utf-8
from __future__ import unicode_literals
import uno
import unohelper
import msgbox
from com.sun.star.beans import PropertyValue
from com.sun.star.awt import Size
from com.sun.star.text.ControlCharacter import PARAGRAPH_BREAK, LINE_BREAK
from com.sun.star.awt.FontWeight import BOLD as FW_BOLD
from com.sun.star.text.TextContentAnchorType import AS_CHARACTER, AT_PARAGRAPH
from com.sun.star.text.WrapTextMode import NONE
import enum
import logging
import time
import subprocess
import json
import tempfile
import os
import uuid
import logging
from utils import *

# # For debug purposes
logging.basicConfig(filename= os.path.join(tempfile.gettempdir(), "insert_formula_lo_write.log"), encoding='utf-8', level=logging.DEBUG)
# logging.disable(logging.CRITICAL)

FORMATS = [
	"svg",
	"png"
]


# The width of the image reported by LO Write is the width of the SVG plus 27 hundredth of a mm
# I don't know why this happens but it messes up size and alignement
PADDING_HACK_CONSTANT_WIDTH  = 26.5  # 1/100mm
PADDING_HACK_CONSTANT_HEIGHT = 26.45 # 1/100mm
FORMULA_FILE = str(uuid.uuid4()) # always write to the same file to avoid building up large number of files in /tmp/

def insert_block_formula(*args):
	insert_formula(block = True)

def insert_inline_formula(*args):
	insert_formula(block = False)



def insert_formula(block):
	settings = get_apso_settings()
	# create temporary file path
	path = os.path.join(tempfile.gettempdir(), FORMULA_FILE)

	# Recover document
	desktop = XSCRIPTCONTEXT.getDesktop()
	doc = desktop.getCurrentComponent()

	# Get graphics provider (loads image files and stuff)
	service_manager  = uno.getComponentContext().getServiceManager() 
	graphic_provider = service_manager.createInstance('com.sun.star.graphic.GraphicProvider')

	# Get cursor
	cursor = doc.CurrentController.ViewCursor	
	char_height = cursor.CharHeight

	# If selection is a GraphicsObject generated by our plugin, then we modify this object
	selection = doc.CurrentController.Selection
	# I wish there was a safer way to do this!
	text_graphic_object = None
	initial_formula = None
	if selection.getImplementationName() == "SwXTextGraphicObject": 
		text_graphic_object = selection
		# When the selection is a TextGraphicsObject, LO gives a default value to CharHeight
		char_height = selection.Anchor.CharHeight
		# TODO: have a way to distinguish formulas from other TextGraphicsShape
		initial_formula = selection.Description


	#######################################
	# LAUNCH PROGRAM
	#######################################
	
	metainfo = launch_maths_preview(settings["MathsPreviewPath"], char_height, path, maths_font = settings.get("MathsFont"), initial_formula = initial_formula, custom_cmd_file = settings.get("CustomCommandFile"))
	if metainfo is None:
		return
	width_px  = metainfo["metrics"]["bbox"]["x_max"] - metainfo["metrics"]["bbox"]["x_min"] 
	height_px = metainfo["metrics"]["bbox"]["y_max"] - metainfo["metrics"]["bbox"]["y_min"]
	if width_px == 0 or height_px == 0:
		return

	#######################################
	# INSERT GRAPHICS
	#######################################

	graphic = create_graphic_object_shape_from_path(doc, graphic_provider, path)

	description = metainfo["formula"]
	is_new_text_graphic_object = text_graphic_object is None
	if is_new_text_graphic_object:
		text_graphic_object = doc.createInstance("com.sun.star.text.TextGraphicObject")

	fill_text_graphic_object_with_shape(text_graphic_object, graphic, width_px, height_px, description)
	
	if block:
		make_block(text_graphic_object)
	else:
		baseline_px = metainfo["metrics"]["baseline"]
		y_min_px    = metainfo["metrics"]["bbox"]["y_min"]
		make_inline(text_graphic_object, y_min_px)


	if is_new_text_graphic_object:
		doc.Text.insertTextContent(cursor, text_graphic_object, False)






def launch_maths_preview(exe_path, char_height, path, maths_font = None, initial_formula = None, custom_cmd_file = None):
	additional_args = []
	if maths_font is not None:
		additional_args.extend(["-m", maths_font,])

	if custom_cmd_file is not None:
		additional_args.extend(["-y", custom_cmd_file,])

	if initial_formula is not None:
		additional_args.extend(["-i", initial_formula,])

	# Start program
	try:
		cmd = [
			exe_path, 
			"-s", str(char_height),
			"-f", "svg", 
			"-d", 
			"-o", path
		] +  additional_args
		logging.debug(" ".join(cmd))
		result = subprocess.run(
			cmd, 
			stdout = subprocess.PIPE, 
			stderr = subprocess.PIPE,
		)
	except FileNotFoundError as e:
		msg_box("Executable could not launch ; check executable path in extension options\n{}".format(str(e)))
		return None

	stdout = result.stdout.decode("utf-8") 
	stderr = result.stderr.decode("utf-8") 
	if result.returncode != 0:
		msg_box(
			"ERROR: maths_preview returned {}\nstdout:\n {}\nstderr:\n {}".format(
				result.returncode,
				stdout, stderr,
			)
		)
		return None
	metainfo = json.loads(stdout)
	return metainfo






def create_graphic_object_shape_from_path(doc, graphic_provider, path,):
	file_url = unohelper.systemPathToFileUrl(path)
	graphic = graphic_provider.queryGraphic((PropertyValue('URL', 0, file_url, 0), ))

	if graphic is None:
		msg_box("No file was returned by the program!")
		return
		

	return graphic





# Desired unit is 1/100mm
# Assume as is standard 96 PPI 
# 1 px = 1 / 96 in = 0.26458333333mm = 26.458333333 1/100mmm
# 1 pt (DTP) = 1 / 72 in = 0.3527778mm = 35.27778 1/100mm
ONE100TH_MM_PER_PX = 26.4583333333


def fill_text_graphic_object_with_shape(text_graphic_object, graphic, width_px, height_px, description = None):
	logging.debug("size px: {} x {}".format(width_px, height_px))
	size = Size(round(width_px * ONE100TH_MM_PER_PX), round(height_px * ONE100TH_MM_PER_PX))
	logging.debug("size 1/100mm (pre-hack): {} x {}".format(size.Width, size.Height))
	size.Width  += PADDING_HACK_CONSTANT_WIDTH 
	size.Height += PADDING_HACK_CONSTANT_HEIGHT
	logging.debug("size 1/100mm (post-hack): {} x {}".format(size.Width, size.Height))

	text_graphic_object.Graphic = graphic
	text_graphic_object.setSize(size)

	if description is not None:
		text_graphic_object.Description = description








def make_inline(text_graphic_object, y_min_px):
	height = text_graphic_object.Size.Height
	logging.debug("Height from Graphics Object {}".format(height))
	text_graphic_object.AnchorType = AS_CHARACTER
	text_graphic_object.VertOrient = 0
	text_graphic_object.VertOrientPosition = - PADDING_HACK_CONSTANT_HEIGHT / 2 + y_min_px * ONE100TH_MM_PER_PX
	logging.debug("text_graphic_object.VertOrientPosition {}".format(text_graphic_object.VertOrientPosition))






def make_block(text_graphic_object):
	text_graphic_object.AnchorType = AT_PARAGRAPH
	text_graphic_object.TextWrap   = NONE











def msg_box(text):
	myBox = msgbox.MsgBox(XSCRIPTCONTEXT.getComponentContext())
	myBox.addButton("OK")
	myBox.renderFromButtonSize()
	myBox.numberOflines = 2
	myBox.show(text,0,"Watching")



def get_apso_settings():
    key = "/InsertFromMathsPreview.Settings"
    reader = getConfigurationAccess(key)
    groupnames = reader.ElementNames
    settings = {}
    for groupname in groupnames:
        group = reader.getByName(groupname)
        props = group.ElementNames
        values = group.getPropertyValues(props)
        settings.update({k: v for k, v in zip(props, values)})

    default_settings = {
    	"MathsPreviewPath"  : "~/bin/maths_preview",
    	"MathsFont"         : None,
    	"CustomCommandFile" : None,
    }

    for k, v in settings.items():
    	if v is None or v.strip() == "":
    		settings[k] = default_settings[k]

    return settings
