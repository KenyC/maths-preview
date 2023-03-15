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
import enum
import logging
import time
import subprocess
import json
import tempfile
import os
import uuid

FORMATS = [
	"svg",
	"png"
]
PATH_EXE = "/home/keny/bin/maths_preview"

def insert_block_formula(*args):
	insert_formula(block = True)

def insert_inline_formula(*args):
	insert_formula(block = False)

def insert_formula(block):
	# create temporary file path
	path = os.path.join(tempfile.gettempdir(), str(uuid.uuid4()))

	# Recover document
	desktop = XSCRIPTCONTEXT.getDesktop()
	doc = desktop.getCurrentComponent()

	# Get graphics provider (loads image files and stuff)
	service_manager  = uno.getComponentContext().getServiceManager() 
	graphic_provider = service_manager.createInstance('com.sun.star.graphic.GraphicProvider')

	# Get cursor
	# cursor = doc.Text.createTextCursor()
	cursor = doc.CurrentController.ViewCursor	
	char_height = cursor.CharHeight

	# Find potential graphics object in selection
	content_enumeration = cursor.getText().createEnumeration()
	while content_enumeration.hasMoreElements():
		print(content_enumeration.nextElement())

	# Start program
	result = subprocess.run([
		PATH_EXE, 
		"-s", str(char_height),
		"-f", "svg", 
		"-d", 
		"-o", path
	], 
		stdout = subprocess.PIPE, 
		stderr = subprocess.PIPE,
	)
	stdout = result.stdout.decode("utf-8") 
	stderr = result.stderr.decode("utf-8") 
	print(stdout)
	print(stderr)
	if result.returncode != 0:
		msg_box(
			"ERROR: maths_preview returned {}\\stdout: {}\\stderr: {}".format(
				result.returncode,
				stdout, stderr,
			)
		)
		return

	metainfo = json.loads(stdout)

	baseline_percentage = None
	if not block:
		baseline_percentage = metainfo["metrics"]["baseline"] / (metainfo["metrics"]["bbox"]["y_max"] - metainfo["metrics"]["bbox"]["y_min"]) 


	description = metainfo["formula"]
	print(metainfo)	

	# Paste image
	add_embedded_image(
		cursor,
		graphic_provider, 
		doc,
		path,
		description,
		baseline_percentage,
	)


def add_embedded_image(cursor, graphic_provider, doc, path, description, baseline_percentage = None, dpi = 1.0, width=None, height=None, paraadjust=None):
	# TODO : dpi should be guessed from buffer
	# scale = 1000 * 2.54 / float(dpi)
	scale = 10 * 2.54 / float(dpi) # this seems like the good ratio, I have no idea why!
	block = baseline_percentage is None


	try:
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
		print("graphic.Size100thMM", graphic.Size100thMM.Width, graphic.Size100thMM.Height)
		print("graphic.SizePixel",   graphic.SizePixel.Width,   graphic.SizePixel.Height)
		graphic_object_shape = doc.createInstance('com.sun.star.drawing.GraphicObjectShape')
		graphic_object_shape.Graphic = graphic
		if width and height:
			size = Size(int(width * scale), int(height * scale))
		elif width:
			size = Size(int(width * scale), int((float(width)/original_size.Width) * original_size.Height * scale))
		elif height:
			size = Size(int((float(height)/original_size.Height) * original_size.Width * scale), int(height * scale))
		else:
			size = Size(int(original_size.Width * scale), original_size.Height * scale)
		graphic_object_shape.setSize(size)
		# doc.Text.insertTextContent(cursor, graphic_object_shape, False)
		text_graphic_object = doc.createInstance("com.sun.star.text.TextGraphicObject")
		text_graphic_object.Graphic = graphic_object_shape.Graphic
		text_graphic_object.setSize(size)
		text_graphic_object.Description = description
		print(baseline_percentage)

		print(original_size.Width, original_size.Height)
		if block:
			print("Block")
			text_graphic_object.AnchorType = AT_PARAGRAPH
		else:
			print("Inline")
			text_graphic_object.AnchorType = AS_CHARACTER
			text_graphic_object.VertOrient = 0
			print("((baseline_percentage + 1) * original_size.Height) * scale", ((baseline_percentage + 1) * original_size.Height) * scale)
			# text_graphic_object.VertOrientPosition = ((baseline_percentage - 1) * original_size.Height) * scale

			"""
			ere             erere
			   +-----------+     ∧
			   |           |     |   (1 + baseline) * height
			   +- - - - - -+ ∧   v
			   |           | |   - baseline * height
			   +-----------+ v


			"""
			text_graphic_object.VertOrientPosition = -  (1 + baseline_percentage) * original_size.Height * scale

		if paraadjust:
			oldparaadjust = cursor.ParaAdjust
			cursor.ParaAdjust = paraadjust
		doc.Text.insertTextContent(cursor, text_graphic_object, False)
		# os.unlink(url)
		if paraadjust:
			cursor.ParaAdjust = oldparaadjust
	except Exception as e:
		print(e)


def msg_box(text):
    myBox = msgbox.MsgBox(XSCRIPTCONTEXT.getComponentContext())
    myBox.addButton("OK")
    myBox.renderFromButtonSize()
    myBox.numberOflines = 2
    myBox.show(text,0,"Watching")

