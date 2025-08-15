#include <stdint.h>

void num2str(uint64_t a0, uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4, uint64_t a5)
{
    a5 = 1717985280;
    a5 = a5 + 1639;
    a5 = a1 * a5;
    a4 = a1 / 2147483648;
    a5 = a5 / 17179869184;
    a5 = a5 - a4;
    a4 = a5 * 4;
    a4 = a4 + a5;
    a4 = a4 * 2;
    a1 = a1 - a4;
    a5 = a5 + 48;
    a1 = a1 + 48;
    ((uint8_t *)(a0))[0] = a5;
    ((uint8_t *)(a0))[1] = a1;
}
