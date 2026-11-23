#pragma once

// Asks for 8 bits per colour channel. Without it Mesa's EGL can hand Qt an
// RGB565 surface, which visibly shifts dark Material surface colours.
void requestTrueColorSurfaces();
