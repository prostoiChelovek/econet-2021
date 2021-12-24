import cv2 as cv
from matplotlib import pyplot as plt
name = 'paper'

img = cv.imread('/home/tima/rubbish/{}/{}0.jpg'.format(name,name))
color = ('b', 'g', 'r')
for i, col in enumerate(color):
    histr = cv.calcHist([img], [i], None, [256], [0, 256])
    plt.plot(histr, color=col)
    plt.xlim([0, 256])
plt.show()
