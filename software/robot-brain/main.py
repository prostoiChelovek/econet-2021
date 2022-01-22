import random as rng

import numpy as np
from cv2 import cv2 as cv

TEST_IMAGE = "./img.png"
RANGE = (
        np.array([25, 100, 95]),
        np.array([105, 140, 145])
        )
CANNY_THRESHOULD = 100


def filter_color(img):
    img = cv.cvtColor(img, cv.COLOR_BGR2YCrCb)
    return cv.inRange(img, *RANGE)


def clean_mask(mask):
    kernel_erode = np.ones((3, 3), np.uint8)
    kernel_dilate = np.ones((6, 6), np.uint8)

    mask = cv.dilate(mask, kernel_dilate, iterations=2)
    mask = cv.erode(mask, kernel_erode, iterations=1)

    return mask


def find_contours(img):
    edges = cv.Canny(img, CANNY_THRESHOULD, CANNY_THRESHOULD * 2)
    contours, hierarchy = cv.findContours(edges, cv.RETR_CCOMP,
                                          cv.CHAIN_APPROX_SIMPLE)
    return contours, hierarchy


def draw_contours(img_size, contours, hierarchy):
    drawing = np.zeros((img_size[0], img_size[1], 3), dtype=np.uint8)

    for i in range(len(contours)):
        color = (rng.randint(0, 256), rng.randint(0, 256), rng.randint(0, 256))
        cv.drawContours(drawing, contours, i, color, 2,
                        cv.LINE_8, hierarchy, 0)

    return drawing


def main():
    img_orig = cv.imread(TEST_IMAGE)

    mask = filter_color(img_orig)
    mask = clean_mask(mask)
    contours, hierarchy = find_contours(mask)

    draw = draw_contours(img_orig.shape, contours, hierarchy)

    cv.imshow("draw", draw)
    cv.waitKey(0)


if __name__ == "__main__":
    main()
