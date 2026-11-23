#include "surface_format.h"

#include <QtGui/QSurfaceFormat>

void requestTrueColorSurfaces()
{
    QSurfaceFormat format = QSurfaceFormat::defaultFormat();
    format.setRedBufferSize(8);
    format.setGreenBufferSize(8);
    format.setBlueBufferSize(8);
    QSurfaceFormat::setDefaultFormat(format);
}
