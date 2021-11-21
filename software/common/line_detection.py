import cv2 as cv

import numpy as np
def nothing(x):
    pass
cv.namedWindow('image')
img = cv.imread('field.png')
gray = cv.cvtColor(img,cv.COLOR_BGR2GRAY)
edges = cv.Canny(gray,50,120)


cv.createTrackbar('minLine','image',0,255,nothing)
cv.createTrackbar('maxLine','image',0,255,nothing)


while(1):
    img_draw = img.copy()
    
    minLineLength = cv.getTrackbarPos('minLine','image')
    maxLineGap = cv.getTrackbarPos('maxLine','image')
    print(minLineLength)
    lines = cv.HoughLinesP(edges,1,np.pi/180,100,minLineLength,maxLineGap)
    print(lines[0])
    for i in range(len(lines)):
        for x1,y1,x2,y2 in lines[i]:
            cv.line(img_draw,(x1,y1),(x2,y2),(0,255,0),2)


    cv.imshow("edges", edges)
    cv.imshow("image", img_draw)
    if cv.waitKey(10) == 27:
        break
    cv.waitKey(1)

cv.destroyAllWindows()
