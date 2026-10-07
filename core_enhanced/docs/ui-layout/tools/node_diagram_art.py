"""Shared SVG drawing and PNG rendering for the current facility diagrams."""
from pathlib import Path
from html import escape
import ctypes as C
import ctypes.util
import math
from render_facility_map import api

ROOT = Path(__file__).resolve().parent.parent / 'node'
BASE = ''
WIDTH, HEIGHT = 1800, 1510
TEMPLATES = []
FONT = 'Malgun Gothic,Noto Sans CJK KR,sans-serif'
parts = []


def text(x, y, value, size=23, fill='#25354b', weight=400, anchor='middle'):
    parts.append(f'<text x="{x}" y="{y}" text-anchor="{anchor}" dominant-baseline="central" font-family="{FONT}" font-size="{size}" font-weight="{weight}" fill="{fill}">{escape(value)}</text>')


def rect(x,y,w,h,fill,stroke='none',radius=18,width=2):
    parts.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{radius}" fill="{fill}" stroke="{stroke}" stroke-width="{width}"/>')


def arrow(tip, tail, color, size=12):
    angle=math.atan2(tip[1]-tail[1],tip[0]-tail[0])
    a=(tip[0]-size*math.cos(angle)+5*math.sin(angle),tip[1]-size*math.sin(angle)-5*math.cos(angle))
    b=(tip[0]-size*math.cos(angle)-5*math.sin(angle),tip[1]-size*math.sin(angle)+5*math.cos(angle))
    points=' '.join(f'{x:.2f},{y:.2f}' for x,y in [tip,a,b])
    parts.append(f'<polygon points="{points}" fill="{color}"/>')


def line(points,color='#91a0b3',both=False,dashed=False):
    dash=' stroke-dasharray="9 8"' if dashed else ''
    coordinates=' '.join(f'{x:.2f},{y:.2f}' for x,y in points)
    parts.append(f'<polyline points="{coordinates}" fill="none" stroke="{color}" stroke-width="3" stroke-linejoin="round"{dash}/>')
    arrow(points[-1],points[-2],color)
    if both:arrow(points[0],points[1],color)


def rasterize():
    cairo=C.CDLL(ctypes.util.find_library('cairo'));rsvg=C.CDLL(ctypes.util.find_library('rsvg-2'))
    fc=C.CDLL(ctypes.util.find_library('fontconfig'));obj=C.CDLL(ctypes.util.find_library('gobject-2.0'))
    ptr=C.c_void_p;config=api(fc,'FcConfigGetCurrent',ptr)()
    add=api(fc,'FcConfigAppFontAddFile',C.c_int,ptr,C.c_char_p)
    for font in ('malgun.ttf','malgunbd.ttf'):
        p=Path('/mnt/c/Windows/Fonts')/font
        if p.exists():assert add(config,str(p).encode())
    error=ptr();handle=api(rsvg,'rsvg_handle_new_from_file',ptr,C.c_char_p,C.POINTER(ptr))(str(ROOT/'svg'/f'{BASE}.svg').encode(),C.byref(error));assert handle
    class Rect(C.Structure):_fields_=[(k,C.c_double) for k in ('x','y','width','height')]
    draw=api(rsvg,'rsvg_handle_render_document',C.c_int,ptr,ptr,C.POINTER(Rect),C.POINTER(ptr))
    for label,x,y,w,h in [('',0,0,WIDTH,HEIGHT)]+[(t['id'],45+i*995,165,920,1510) for i,t in enumerate(TEMPLATES)]:
        surface=api(cairo,'cairo_image_surface_create',ptr,C.c_int,C.c_int,C.c_int)(0,w,h)
        cr=api(cairo,'cairo_create',ptr,ptr)(surface)
        api(cairo,'cairo_translate',None,ptr,C.c_double,C.c_double)(cr,-x,-y)
        assert draw(handle,cr,C.byref(Rect(0,0,WIDTH,HEIGHT)),C.byref(error))
        suffix='-'+label if label else ''
        assert api(cairo,'cairo_surface_write_to_png',C.c_int,ptr,C.c_char_p)(surface,str(ROOT/f'{BASE}{suffix}.png').encode())==0
        api(cairo,'cairo_destroy',None,ptr)(cr);api(cairo,'cairo_surface_destroy',None,ptr)(surface)
    api(obj,'g_object_unref',None,ptr)(handle)
