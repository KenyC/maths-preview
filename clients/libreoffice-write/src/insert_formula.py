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
from utils import *

FORMATS = [
	"svg",
	"png"
]





def insert_block_formula(*args):
	insert_formula(block = True)

def insert_inline_formula(*args):
	insert_formula(block = False)



def insert_formula(block):
	settings = get_apso_settings()
	# create temporary file path
	path = os.path.join(tempfile.gettempdir(), str(uuid.uuid4()))

	# Recover document
	desktop = XSCRIPTCONTEXT.getDesktop()
	doc = desktop.getCurrentComponent()

	# Get graphics provider (loads image files and stuff)
	service_manager  = uno.getComponentContext().getServiceManager() 
	graphic_provider = service_manager.createInstance('com.sun.star.graphic.GraphicProvider')

	# Get cursor
	cursor = doc.CurrentController.ViewCursor	
	char_height = cursor.CharHeight

	#######################################
	# LAUNCH PROGRAM
	#######################################
	
	metainfo = launch_maths_preview(settings["MathsPreviewPath"], char_height, path, maths_font = settings.get("MathsFont"))
	assert(metainfo is not None)

	#######################################
	# INSERT GRAPHICS
	#######################################

	graphic_object_shape = create_graphic_object_shape_from_path(doc, graphic_provider, path)


	description = metainfo["formula"]
	text_graphic_object  = create_text_graphic_object(doc, graphic_object_shape, description)
	
	if block:
		make_block(text_graphic_object)
	else:
		baseline_percentage = metainfo["metrics"]["baseline"] / (metainfo["metrics"]["bbox"]["y_max"] - metainfo["metrics"]["bbox"]["y_min"]) 
		make_inline(text_graphic_object, baseline_percentage)

	doc.Text.insertTextContent(cursor, text_graphic_object, False)






def launch_maths_preview(exe_path, char_height, path, maths_font = None):
	additional_args = []
	if maths_font is not None:
		additional_args.extend(["-m", maths_font,])

	# Start program
	try:
		result = subprocess.run([
			exe_path, 
			"-s", str(char_height),
			"-f", "svg", 
			"-d", 
			"-o", path
		] +  additional_args, 
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
			"ERROR: maths_preview returned {}\\stdout: {}\\stderr: {}".format(
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
		
	if graphic.SizePixel is None:
		# Then we're likely dealing with vector graphics. Then we try to
		# get the "real" size, which is enough information to
		# determine the aspect ratio
		original_size = graphic.Size100thMM
	else:
		original_size = graphic.SizePixel

	graphic_object_shape = doc.createInstance('com.sun.star.drawing.GraphicObjectShape')
	graphic_object_shape.Graphic = graphic

	return graphic_object_shape







def create_text_graphic_object(doc, graphic_object_shape, description = None, dpi = 1.0):
	scale = 10 * 2.54 / float(dpi) # this seems like the good ratio, I have no idea why!
	original_size = graphic_object_shape.Graphic.SizePixel
	size = Size(int(original_size.Width * scale), original_size.Height * scale)

	text_graphic_object = doc.createInstance("com.sun.star.text.TextGraphicObject")
	text_graphic_object.Graphic = graphic_object_shape.Graphic
	text_graphic_object.setSize(size)

	if description is not None:
		text_graphic_object.Description = description

	return text_graphic_object







def make_inline(text_graphic_object, baseline_percentage):
	height = text_graphic_object.Size.Height
	text_graphic_object.AnchorType = AS_CHARACTER
	text_graphic_object.VertOrient = 0
	text_graphic_object.VertOrientPosition = -  (1 + baseline_percentage) * height






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
    	"MathsPreviewPath" : "~/bin/maths_preview",
    	"MathsFont"        : None,
    }

    for k, v in settings.items():
    	if v is None or v.strip() == "":
    		settings[k] = default_settings[k]

    return settings
