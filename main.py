import pystray
from PIL import Image, ImageDraw

def on_exit_callback(icon, item):
    print("Exiting...")

def on_click_callback(icon, item):
    print("Hello, World!")

# 아이콘 이미지 생성
image = Image.new("RGB", (30, 30), (255, 255, 255))
draw = ImageDraw.Draw(image)
draw.rectangle([(0, 0), (30, 30)], fill=(255, 0, 0))  # 예시로 빨간색 사각형

# 트레이 아이콘 설정
icon = pystray.Icon("example", image, "Hello, World!")
icon.menu = pystray.Menu(
    pystray.MenuItem("Say Hello", on_click_callback),
    pystray.MenuItem("Exit", on_exit_callback)
)

# 아이콘을 트레이에 추가
icon.run()