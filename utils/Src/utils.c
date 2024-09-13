
/*
******************************************************************************
* INCLUDES
******************************************************************************
*/
#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

/*
******************************************************************************
* FFI
******************************************************************************
*/
extern void ffi_log(const uint8_t* s, size_t len);

/*
******************************************************************************
* GLOBAL DEFINES
******************************************************************************
*/

#if (USE_LOGGER == LOGGER_ON)
#define MAX_HEX_STR         4
#define MAX_HEX_STR_LENGTH  128
#endif /* #if USE_LOGGER == LOGGER_ON */

/*
******************************************************************************
* GLOBAL VARIABLES
******************************************************************************
*/

#if (USE_LOGGER == LOGGER_ON)
char hexStr[MAX_HEX_STR][MAX_HEX_STR_LENGTH];
uint8_t hexStrIdx = 0;
#endif /* #if USE_LOGGER == LOGGER_ON */

/*
******************************************************************************
* FUNCTIONS DEFINITION
******************************************************************************
*/

int logString(const char* format, ...)
{
    #define LOG_BUFFER_SIZE 256
    char buf[LOG_BUFFER_SIZE];
    va_list argptr;
    va_start(argptr, format);
    int cnt = vsnprintf(buf, LOG_BUFFER_SIZE, format, argptr);
    va_end(argptr);

    ffi_log((uint8_t*)buf, strlen(buf));
    return cnt;
}

char* hex2Str(unsigned char * data, size_t dataLen)
{
#if (USE_LOGGER == LOGGER_ON)
    const char * hex = "0123456789ABCDEF";
    
    unsigned char * pin  = data;
    char *          pout = hexStr[hexStrIdx];

    uint8_t idx = hexStrIdx;

    if( dataLen > (MAX_HEX_STR_LENGTH/2) )
    {
        dataLen = (MAX_HEX_STR_LENGTH/2) - 1;
    }

    for(uint32_t i = 0; i < dataLen; i++)
    {
        *pout++ = hex[(*pin>>4) & 0x0F];
        *pout++ = hex[(*pin++)  & 0x0F];
    }
    *pout = 0;

    hexStrIdx++;
    hexStrIdx %= MAX_HEX_STR;
    
    return hexStr[idx];
#else
    return NULL;
#endif /* #if USE_LOGGER == LOGGER_ON */
}
