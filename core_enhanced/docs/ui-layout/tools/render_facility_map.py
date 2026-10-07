"""Rasterize the authored SVG and its wing views using system librsvg/Cairo.

This renders code-native vector artwork; no generated-image service is needed.
"""
from pathlib import Path
import ctypes as C
import ctypes.util

ROOT = Path(__file__).resolve().parent.parent / 'node'


def api(lib, name, result, *args):
    fn = getattr(lib, name)
    fn.restype, fn.argtypes = result, list(args)
    return fn


def render():
    cairo = C.CDLL(ctypes.util.find_library('cairo'))
    rsvg = C.CDLL(ctypes.util.find_library('rsvg-2'))
    obj = C.CDLL(ctypes.util.find_library('gobject-2.0'))
    fc = C.CDLL(ctypes.util.find_library('fontconfig'))
    ptr, number = C.c_void_p, C.c_double
    config = api(fc, 'FcConfigGetCurrent', ptr)()
    add_font = api(fc, 'FcConfigAppFontAddFile', C.c_int, ptr, C.c_char_p)
    # Use existing local fonts without copying them or changing system configuration.
    for name in ('malgun.ttf', 'malgunbd.ttf'):
        path = Path('/mnt/c/Windows/Fonts') / name
        if path.exists():
            assert add_font(config, str(path).encode())
    create = api(cairo, 'cairo_image_surface_create', ptr, C.c_int, C.c_int, C.c_int)
    context = api(cairo, 'cairo_create', ptr, ptr)
    scale = api(cairo, 'cairo_scale', None, ptr, number, number)
    translate = api(cairo, 'cairo_translate', None, ptr, number, number)
    save = api(cairo, 'cairo_surface_write_to_png', C.c_int, ptr, C.c_char_p)
    destroy_context = api(cairo, 'cairo_destroy', None, ptr)
    destroy_surface = api(cairo, 'cairo_surface_destroy', None, ptr)
    release = api(obj, 'g_object_unref', None, ptr)
    load = api(rsvg, 'rsvg_handle_new_from_file', ptr, C.c_char_p, C.POINTER(ptr))

    class Rect(C.Structure):
        _fields_ = [(key, number) for key in ('x', 'y', 'width', 'height')]

    draw = api(rsvg, 'rsvg_handle_render_document', C.c_int, ptr, ptr, C.POINTER(Rect), C.POINTER(ptr))
    error = ptr()
    handle = load(str(ROOT / 'svg' / 'facility-map-review-v2.svg').encode(), C.byref(error))
    if not handle:
        raise RuntimeError('Unable to load facility SVG')
    views = {'': (0, 0, 2800, 3010), 'A': (75, 275, 885, 2585),
             'B': (975, 275, 885, 2585), 'C': (1875, 275, 885, 2585)}
    try:
        for wing, (x, y, width, height) in views.items():
            surface = create(0, width, height)  # CAIRO_FORMAT_ARGB32
            cr = context(surface)
            try:
                scale(cr, 1, 1)
                translate(cr, -x, -y)
                if not draw(handle, cr, C.byref(Rect(0, 0, 2800, 3010)), C.byref(error)):
                    raise RuntimeError('SVG rendering failed')
                suffix = '-' + wing if wing else ''
                target = ROOT / f'facility-map-review-v2{suffix}.png'
                if save(surface, str(target).encode()) != 0:
                    raise RuntimeError(f'PNG write failed: {target.name}')
                print(target.name)
            finally:
                destroy_context(cr)
                destroy_surface(surface)
    finally:
        release(handle)


if __name__ == '__main__':
    render()
