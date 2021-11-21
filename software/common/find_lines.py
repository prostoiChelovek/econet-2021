import random as rng

import cv2 as cv
import numpy as np

rng.seed(12345)

TEST_IMAGE = 'field.png'
COLOR_MIN = (150, 110, 130)
COLOR_MAX = (240, 125, 150)
CANNY_THRESHOULD = 100


def preprocess_image(img):
	return cv.cvtColor(img, cv.COLOR_BGR2LAB)


def clean_mask(mask):
	kernel_erote = np.ones((5, 5),np.uint8)
	erosion = cv.erode(mask, kernel_erote, iterations = 1)

	kernel_dilate = np.ones((3, 3),np.uint8)
	dilation = cv.dilate(erosion, kernel_dilate, iterations = 2)

	return dilation
	

def fiter_color(img):
	return cv.inRange(img, COLOR_MIN, COLOR_MAX)


def find_contours(img):
	edges = cv.Canny(img, CANNY_THRESHOULD, CANNY_THRESHOULD * 2)
	cv.imshow('edges', edges)
	contours, hierarchy = cv.findContours(edges, cv.RETR_CCOMP , cv.CHAIN_APPROX_SIMPLE)
	return contours, hierarchy


def draw_contours(img_size, contours, hierarchy):
	drawing = np.zeros((img_size[0], img_size[1], 3), dtype=np.uint8)

	for i in range(len(contours)):
		color = (rng.randint(0,256), rng.randint(0,256), rng.randint(0,256))
		cv.drawContours(drawing, contours, i, color, 2, cv.LINE_8, hierarchy, 0)

	return drawing


def main():
	img_raw = cv.imread(TEST_IMAGE)
	img = preprocess_image(img_raw)
	mask = fiter_color(img)
	mask = clean_mask(mask)
	contours, hierarchy = find_contours(mask)
	contours_img = draw_contours(mask.shape, contours, hierarchy)

	cv.imshow('mask', mask)
	cv.imshow('contours',contours_img)
	cv.waitKey(0)



if __name__ == '__main__':
	main() 

