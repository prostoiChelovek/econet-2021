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
                           
    return sort_hist(hist[:NUM_SAMPLES])

    
    ax[j][k].set_xticklabels(histx)
    plt.setp(ax[j][k].get_xticklabels(), rotation = r, horizontalalignment='right', fontsize='x-small')


fig , ax = plt.subplots(nrows = 3, ncols = 3, figsize = (25,25))
fig.suptitle('rubbish')

images = list(map(load_image, NAMES))

for k, img in enumerate(images):
    for j, col in enumerate(('b', 'g', 'r')):
        hist = create_hist(img, j)

        histx = [str(x[1]) for x in hist]
        histy = [x[0] for x in hist]
        
        subplot(j, k, col, histx, histy, name=NAMES[k], r=50)
    
plt.show()