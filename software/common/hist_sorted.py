import cv2 as cv
import numpy as np
from matplotlib import pyplot as plt

NAMES = ('paper', 'tin', 'bottle')
NUM_SAMPLES = 40
LABLE_ROTATION = 50

def load_image (name, num=0): 
    return cv.imread(f'/home/tima/rubbish/{name}/{name}{num}.jpg')

def sort_hist(hist):
    # [(amount, value)]
    hist_map = [(y[0], x) for x, y in enumerate(hist)]
    return sorted(hist_map, key=lambda x: x[0], reverse=True)


def create_hist(img, channel):     
    hist = cv.calcHist(images=[img],
                       channels=[channel],
                       mask=None,
                       histSize=[256],
                       ranges=[0, 256])
                           
    return sort_hist(hist)[:NUM_SAMPLES]


def create_hists(images):
    return  [ 
        [
            create_hist(img, channel) 
            for channel in range(3)
        ] for img in images
    ]
    

def create_plot_data(hists):
    res = []
    for image_hists in hists:
        image_data = []
        for hist in image_hists:
            histx = [str(x[1]) for x in hist]
            histy = [x[0] for x in hist]

            image_data.append((histx, histy))
        res.append(image_data)

    return res


def plot_hist(axis, histx, histy, color):
    axis.plot(histx, histy, color)
    axis.set_xticklabels(histx)
    plt.setp(axis.get_xticklabels(),
             rotation=LABLE_ROTATION,
             horizontalalignment='right',
             fontsize='x-small')


def plot(axes, plot_data):
    for col, image_data in enumerate(plot_data):
        for row, (x, y) in enumerate(image_data):
            axis = axes[row][col]
            color = ('b', 'g', 'r')[row]  
            plot_hist(axis, x, y, color)


def main():
    fig, ax = plt.subplots(nrows=3, ncols=3, figsize=(25, 25))
    fig.suptitle('rubbish')

    images = list(map(load_image, NAMES))
    hists = create_hists(images)
    plot_data = create_plot_data(hists)
    plot(ax, plot_data)

    plt.show()


if __name__  == "__main__":
    main()