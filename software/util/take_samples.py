from cv2 import cv2 as cv

from os import listdir
from os.path import isfile, join

IMAGES_DIR = "./images"
SAMPLES_DIRS = {
    "t": IMAGES_DIR + "/tin",
    "b": IMAGES_DIR + "/bottle",
    "p": IMAGES_DIR + "/paper",
    "f": IMAGES_DIR + "/field"
}


def get_next_image_index(directory):
    files = [f for f in listdir(directory) if isfile(join(directory, f))]
    names = map(lambda x: x.split(".")[0], files)
    indexes = map(int, filter(lambda x: x.isdigit(), names))
    return max(indexes, default=-1) + 1


cap = cv.VideoCapture(2)

while cap.isOpened():
    _, frame = cap.read()

    cv.imshow("f", frame)

    key = cv.waitKey(1)

    if key in range(0x110000) and chr(key) in SAMPLES_DIRS.keys():
        directory = SAMPLES_DIRS[chr(key)]
        idx = get_next_image_index(directory)
        path = f"{directory}/{idx}.png"
        cv.imwrite(path, frame)
    elif key == 27:  # esc
        break
