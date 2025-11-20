
/******************************************************************************
  * @attention
  *
  * COPYRIGHT 2018 STMicroelectronics, all rights reserved
  *
  * Unless required by applicable law or agreed to in writing, software
  * distributed under the License is distributed on an "AS IS" BASIS,
  * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied,
  * AND SPECIFICALLY DISCLAIMING THE IMPLIED WARRANTIES OF MERCHANTABILITY,
  * FITNESS FOR A PARTICULAR PURPOSE, AND NON-INFRINGEMENT.
  * See the License for the specific language governing permissions and
  * limitations under the License.
  *
******************************************************************************/


/*
 *      PROJECT:   ST25R95 firmware
 *      $Revision: $
 *      LANGUAGE:  ISO C99
 */

/*! \file st25r95_com_spi.c
 *
 *  \author 
 *
 *  \brief Implementation of ST25R95 communication.
 *
 */

/*
 ******************************************************************************
 * INCLUDES
 ******************************************************************************
 */
 
#include "st25r95_com.h"
#include "st25r95.h"
#include "string.h"
#include "rfal_nfcb.h"
#include "rfal_nfcf.h"
#include "rfal_rf.h"
#include "rfal_utils.h"

/*
 ******************************************************************************
 * ENABLE SWITCH
 ******************************************************************************
 */
 
 #if !(ST25R95_INTERFACE_UART) /* ST25R95_INTERFACE_SPI */

/*
******************************************************************************
* GLOBAL MACROS
******************************************************************************
*/

#define ST25R95_POLL_FLAG_DATA_CAN_BE_READ_Pos           (3U)                                                                                       /*!< SPI poll flag bit 3: Data can be read when set */
#define ST25R95_POLL_FLAG_DATA_CAN_BE_READ_Msk           (0x1U << ST25R95_POLL_FLAG_DATA_CAN_BE_READ_Pos)                                           /*!< Mask 0x08 */  
#define ST25R95_POLL_FLAG_DATA_CAN_BE_READ               ST25R95_POLL_FLAG_DATA_CAN_BE_READ_Msk                                                     /*!< 0x08 */
#define ST25R95_POLL_DATA_CAN_BE_READ(Flags)             (((Flags) & ST25R95_POLL_FLAG_DATA_CAN_BE_READ_Msk) == ST25R95_POLL_FLAG_DATA_CAN_BE_READ) /*!< SPI read poll flag test */
#define ST25R95_POLL_FLAG_DATA_CAN_BE_SEND_Pos           (2U)                                                                                       /*!< SPI poll flag bit 2: Data can be send when set */
#define ST25R95_POLL_FLAG_DATA_CAN_BE_SEND_Msk           (0x1U << ST25R95_POLL_FLAG_DATA_CAN_BE_SEND_Pos)                                           /*!< Mask 0x04 */
#define ST25R95_POLL_FLAG_DATA_CAN_BE_SEND               ST25R95_POLL_FLAG_DATA_CAN_BE_SEND_Msk                                                     /*!< 0x04 */
#define ST25R95_POLL_DATA_CAN_BE_SEND(Flags)             (((Flags) & ST25R95_POLL_FLAG_DATA_CAN_BE_SEND_Msk) == ST25R95_POLL_FLAG_DATA_CAN_BE_SEND) /*!< SPI send poll flag test*/


/*
 ******************************************************************************
 * LOCAL VARIABLES
 ******************************************************************************
 */

static uint8_t EchoCommand[1] = {ST25R95_COMMAND_ECHO};
static uint8_t Idle[] = {ST25R95_COMMAND_IDLE, 0x0E, 0x0A, 0x21, 0x00, 0x38, 0x01, 0x18, 0x00, 0x20, 0x60, 0x60, 0x74, 0x84, 0x3F, 0x00};

/*
 ******************************************************************************
 * GLOBAL VARIABLES
 ******************************************************************************
 */
 
st25r95SPIRxContext st25r95SPIRxCtx;

/*
******************************************************************************
* GLOBAL FUNCTIONS
******************************************************************************
*/

/*******************************************************************************/
ReturnCode st25r95SPIPollRead(uint32_t timeout)
{
    ReturnCode retCode = RFAL_ERR_NONE;   
    
    if (!platformWaitIrqOutFallingEdge(timeout))
    {
        retCode = RFAL_ERR_TIMEOUT;
    }
    
    return (retCode);
}

/*******************************************************************************/
ReturnCode st25r95SPIPollSend(void)
{
    ReturnCode retCode = RFAL_ERR_NONE;
    
    
    if (!platformSpiPollSend())
    {
        retCode = RFAL_ERR_TIMEOUT;
    }
    return (retCode);
}

/*******************************************************************************/
ReturnCode st25r95SPISendCommandTypeAndLen(uint8_t *cmd, uint8_t *resp, uint16_t respBuffLen)
{
    ReturnCode retCode = RFAL_ERR_NONE;
    uint32_t len;
    
    if (respBuffLen < 2)
    {
        retCode = RFAL_ERR_NOMEM;
    }
    else
    {
        resp[ST25R95_CMD_RESULT_OFFSET] = ST25R95_ERRCODE_COMERROR;
        resp[ST25R95_CMD_LENGTH_OFFSET] = 0x00;
        
        /* 1 - Send the  command */
        platformSpiSendCmd(cmd[0], cmd + 2, cmd[ST25R95_CMD_LENGTH_OFFSET], false);
        #if ST25R95_DEBUG
        platformLog("[%10d] >>>> %s\r\n", platformGetSysTick(), hex2Str(cmd, cmd[ST25R95_CMD_LENGTH_OFFSET] + 2));
        #endif /* ST25R95_DEBUG */
        
        /* 2 - Poll the ST25R95 until it is ready to transmit */
        retCode = st25r95SPIPollRead(ST25R95_CONTROL_POLL_TIMEOUT);
        
        if (retCode == RFAL_ERR_NONE) 
        {
            len = platformSpiRead(&resp[ST25R95_CMD_RESULT_OFFSET], &resp[ST25R95_CMD_DATA_OFFSET], respBuffLen - ST25R95_CMD_DATA_OFFSET);
            resp[ST25R95_CMD_LENGTH_OFFSET] = (uint8_t) (len & 0xFFU);
            resp[ST25R95_CMD_RESULT_OFFSET] |= (uint8_t) ((len >> 3U) & 0x60U);
            if (respBuffLen < (len + 2))
            {
                st25r95FlushChipSPIBuffer();
                retCode = RFAL_ERR_NOMEM;
            }
            #if ST25R95_DEBUG
            platformLog("[%10d] <<<< %s\r\n", platformGetSysTick(), hex2Str(resp, len + 2));
            #endif /* ST25R95_DEBUG */
        }
        else
        {
            st25r95FlushChipSPIBuffer();
            retCode = RFAL_ERR_SYSTEM;
        }
    }
    return (retCode);
}

/*******************************************************************************/
ReturnCode st25r95SPICommandEcho(void)
{
    ReturnCode retCode = RFAL_ERR_NONE;
    
    /* 0 - Poll the ST25R95 to make sure data can be send */
    /* Used only in cas of ECHO Command as this command is sent just after the ST25R95 reset */
    retCode = st25r95SPIPollSend();
    
    if (retCode == RFAL_ERR_NONE)
    {
        /* 1 - Send the echo command */
        platformSpiSendCmd(EchoCommand[0], NULL, 0, false);
#if ST25R95_DEBUG        
        platformLog("[%10d] >>>> %2.2x\r\n", platformGetSysTick(), ST25R95_COMMAND_ECHO);
#endif /* ST25R95_DEBUG */
        
        /* 2 - Poll the ST25R95 until it is ready to transmit */
        retCode = st25r95SPIPollRead(ST25R95_CONTROL_POLL_TIMEOUT);
        
        /* 3 - Read echo response */
        if (retCode == RFAL_ERR_NONE) 
        {
            if (!platformSpiReadEcho())
            {
#if ST25R95_DEBUG  
                platformLog("%s: unexepected echo response: %2.2x\r\n", __FUNCTION__, respBuffer[ST25R95_CMD_RESULT_OFFSET]);
#endif /* ST25R95_DEBUG */
                st25r95FlushChipSPIBuffer();
                retCode = RFAL_ERR_SYSTEM;
            }
        }
    }
    #if RFAL_FEATURE_LISTEN_MODE
    st25r95SPIRxCtx.inListen = false;
    #endif /* RFAL_FEATURE_LISTEN_MODE */
    
#if ST25R95_DEBUG  
    platformLog("%s: retCode: %2.2x\r\n", __FUNCTION__, retCode);
#endif /* ST25R95_DEBUG */   
    return (retCode);
}

/*******************************************************************************/
void st25r95SPISendData(uint8_t *buf, uint8_t bufLen, uint8_t protocol, uint32_t flags)
{
    uint8_t cmd = ST25R95_COMMAND_SENDRECV;
 
#if ST25R95_DEBUG
    platformLog("[%10d] DATA >>>> %s", platformGetSysTick(), hex2Str(buf, bufLen));  
#endif /* ST25R95_DEBUG */

    if (protocol == ST25R95_PROTOCOL_CE_ISO14443A)
    {
        /* Card Emulation mode */
        cmd = ST25R95_COMMAND_SEND;
    }
    platformSpiSendCmd(cmd, buf, bufLen, (protocol == ST25R95_PROTOCOL_ISO14443A) && ((flags & RFAL_TXRX_FLAGS_NFCIP1_ON) == RFAL_TXRX_FLAGS_NFCIP1_ON));
}

/*******************************************************************************/
void st25r95SPIPrepareRx(uint8_t protocol, uint8_t *rxBuf, uint16_t rxBufLen, uint16_t *rxRcvdLen, uint32_t flags, uint8_t *additionalRespBytes)
{
    st25r95SPIRxCtx.protocol            = protocol;
    st25r95SPIRxCtx.rxBuf               = rxBuf;
    st25r95SPIRxCtx.rxBufLen            = rxBufLen;
    st25r95SPIRxCtx.rxRcvdLen           = rxRcvdLen;
    st25r95SPIRxCtx.rmvCRC              = ((flags & RFAL_TXRX_FLAGS_CRC_RX_KEEP) != RFAL_TXRX_FLAGS_CRC_RX_KEEP);
    st25r95SPIRxCtx.NFCIP1              = ((protocol == ST25R95_PROTOCOL_ISO14443A) && ((flags & RFAL_TXRX_FLAGS_NFCIP1_ON) == RFAL_TXRX_FLAGS_NFCIP1_ON));
    st25r95SPIRxCtx.additionalRespBytes = additionalRespBytes;
}

/*******************************************************************************/
ReturnCode st25r95SPICompleteRx(void)
{
    uint8_t Result;
#if ST25R95_DEBUG
    uint8_t initialResult;
    int16_t initialLen;
#endif /* ST25R95_DEBUG */
    uint16_t len;
    uint16_t rcvdLen;
    rfalBitRate rxBr;
    ReturnCode retCode = RFAL_ERR_NONE;
    uint16_t additionalRespBytesNb = 1;
    uint8_t buf[ST25R95_COMMUNICATION_BUFFER_SIZE];
    uint16_t offset = 0;
    
    
#if ST25R95_DEBUG   
    RFAL_NO_WARNING(initialResult); /* debug purposes */
    RFAL_NO_WARNING(initialLen);    /* debug purposes */
#endif /* ST25R95_DEBUG */
    
    len = platformSpiRead(&Result, buf, ST25R95_COMMUNICATION_BUFFER_SIZE);
#if ST25R95_DEBUG
    initialResult = Result;
    initialLen = len;
#endif /* ST25R95_DEBUG */

    rcvdLen = 0;
        
    switch (Result)
    {
        case ST25R95_ERRCODE_NONE:
        case ST25R95_ERRCODE_FRAMEOKADDITIONALINFO:
        case ST25R95_ERRCODE_RESULTSRESIDUAL:
            break;
        case ST25R95_ERRCODE_COMERROR:
            retCode = RFAL_ERR_INTERNAL;
            break;
        case ST25R95_ERRCODE_FRAMEWAITTIMEOUT:
            retCode = RFAL_ERR_TIMEOUT;
            break;
        case ST25R95_ERRCODE_OVERFLOW:
            retCode = RFAL_ERR_HW_OVERRUN;
            break;
        case ST25R95_ERRCODE_INVALIDSOF:
        case ST25R95_ERRCODE_RECEPTIONLOST:
        case ST25R95_ERRCODE_FRAMING:
        case ST25R95_ERRCODE_EGT:
        case ST25R95_ERRCODE_61_SOF:
        case ST25R95_ERRCODE_63_SOF_HIGH:
        case ST25R95_ERRCODE_65_SOF_LOW:
        case ST25R95_ERRCODE_66_EGT:
        case ST25R95_ERRCODE_67_TR1TOOLONG:
        case ST25R95_ERRCODE_68_TR1TOOSHORT:
            retCode = RFAL_ERR_FRAMING;
            break; 
        case ST25R95_ERRCODE_62_CRC:   
            retCode = RFAL_ERR_CRC;
            break; 
        case ST25R95_ERRCODE_NOFIELD:
            retCode = RFAL_ERR_LINK_LOSS;
            break;
        default:
            retCode = RFAL_ERR_SYSTEM;
            break;
    }
    
    if ((retCode != RFAL_ERR_NONE) && (len != 0))
    {
        st25r95FlushChipSPIBuffer();
        len = 0;
    }
    
    
    /* In ISO14443A 106kbps 2 additional bytes of collision information are provided */
    rfalGetBitRate( NULL, &rxBr );
    if( (st25r95SPIRxCtx.protocol == ST25R95_PROTOCOL_ISO14443A) && (rxBr == RFAL_BR_106) )
    {
        additionalRespBytesNb += 2;
    }
    
    
    /* read the frame */
    do {
        if (len == 0)
        {
            additionalRespBytesNb = 0;
            break;
        }
        if (len < additionalRespBytesNb)
        {
            /* Flush ST25R95 fifo */
            st25r95FlushChipSPIBuffer();
            retCode = RFAL_ERR_SYSTEM;
            break;
        }
        len -= additionalRespBytesNb;
        if ((Result == ST25R95_ERRCODE_RESULTSRESIDUAL) && (st25r95SPIRxCtx.protocol == ST25R95_PROTOCOL_ISO14443A))
        {
                st25r95SPIRxCtx.rmvCRC = false;
        }
        if ((st25r95SPIRxCtx.rmvCRC) && (st25r95SPIRxCtx.protocol != ST25R95_PROTOCOL_ISO18092))
        {
            if (len < 2)
            {
                /* Flush ST25R95 fifo */
                st25r95FlushChipSPIBuffer();
                additionalRespBytesNb = 0;
                retCode = RFAL_ERR_SYSTEM;
                break;
            }
            len -= 2;
        }
        if ((st25r95SPIRxCtx.NFCIP1) && (len >= 1))
        {
            st25r95SPIRxCtx.NFCIP1_SoD[0] = buf[offset++];
            len -= 1;
        }
        if ((len > st25r95SPIRxCtx.rxBufLen) ||
            ((st25r95SPIRxCtx.protocol == ST25R95_PROTOCOL_ISO18092) && ((len + 1U) > st25r95SPIRxCtx.rxBufLen)) || /* Need one extra byte room to prepend Len byte in rxBuf in case of Felica */
            ((!st25r95SPIRxCtx.rmvCRC) && (st25r95SPIRxCtx.protocol == ST25R95_PROTOCOL_ISO18092) && ((len + 3U) > st25r95SPIRxCtx.rxBufLen))) /* same + 2 extra bytes room to append CRC */
        {
            /* Flush ST25R95 fifo */
            st25r95FlushChipSPIBuffer();
            additionalRespBytesNb = 0;
            retCode = RFAL_ERR_NOMEM;
            break;
        }
        rcvdLen = len;
        if (len != 0)
        {
            if (st25r95SPIRxCtx.protocol == ST25R95_PROTOCOL_ISO18092)
            {
                memcpy(&st25r95SPIRxCtx.rxBuf[RFAL_NFCF_LENGTH_LEN], &buf[offset], len);
                offset += len;
                rcvdLen += RFAL_NFCF_LENGTH_LEN;
                len += RFAL_NFCF_LENGTH_LEN;
                st25r95SPIRxCtx.rxBuf[0] = (uint8_t)(rcvdLen & 0xFFU);
            }
            else
            {
                memcpy(st25r95SPIRxCtx.rxBuf, &buf[offset], len);
                offset += len;
            }
        }
        if ((st25r95SPIRxCtx.rmvCRC) && (st25r95SPIRxCtx.protocol != ST25R95_PROTOCOL_ISO18092))
        {
            memcpy(st25r95SPIRxCtx.BufCRC, &buf[offset], 2);
            offset += 2;
        }
        memcpy(st25r95SPIRxCtx.additionalRespBytes, &buf[offset], additionalRespBytesNb);
     
        /* check collision and CRC error */
        switch (st25r95SPIRxCtx.protocol)
        {
            case (ST25R95_PROTOCOL_ISO15693):
                retCode = ST25R95_IS_PROT_ISO15693_COLLISION_ERR(st25r95SPIRxCtx.additionalRespBytes[0]) ? RFAL_ERR_RF_COLLISION : (ST25R95_IS_PROT_ISO15693_CRC_ERR(st25r95SPIRxCtx.additionalRespBytes[0]) ? RFAL_ERR_CRC : retCode);
                break;
            case (ST25R95_PROTOCOL_ISO14443A):
                retCode = (Result == ST25R95_ERRCODE_RESULTSRESIDUAL) ? ((ReturnCode)(RFAL_ERR_INCOMPLETE_BYTE + ((st25r95SPIRxCtx.additionalRespBytes[0] & 0xFU) % 8U))) : (ST25R95_IS_PROT_ISO14443A_COLLISION_ERR(st25r95SPIRxCtx.additionalRespBytes[0]) ? RFAL_ERR_RF_COLLISION : (ST25R95_IS_PROT_ISO14443A_PARITY_ERR(st25r95SPIRxCtx.additionalRespBytes[0]) ? RFAL_ERR_PAR : (ST25R95_IS_PROT_ISO14443A_CRC_ERR(st25r95SPIRxCtx.additionalRespBytes[0]) ? RFAL_ERR_CRC : retCode)));
                break;
            case (ST25R95_PROTOCOL_ISO14443B):
                if (ST25R95_IS_PROT_ISO14443B_CRC_ERR(st25r95SPIRxCtx.additionalRespBytes[0]))
                {
                    retCode = RFAL_ERR_CRC;
                }
                break;
            case (ST25R95_PROTOCOL_ISO18092):
                if (ST25R95_IS_PROT_ISO18092_CRC_ERR(st25r95SPIRxCtx.additionalRespBytes[0]))
                {
                    retCode = RFAL_ERR_CRC;
                }
                break;                        
            default:
                break;
        }
    } while (0);
   
#if ST25R95_DEBUG
    platformLog("[%10d] DATA <<<<(0x%2.2X%2.2X) %s%s", platformGetSysTick(), initialResult, initialLen, (st25r95SPIRxCtx.NFCIP1) ? hex2Str(st25r95SPIRxCtx.NFCIP1_SoD, 1) : "", (rcvdLen == len) ? hex2Str(st25r95SPIRxCtx.rxBuf, (rcvdLen)): "<error>");
    if ((st25r95SPIRxCtx.rmvCRC) && (additionalRespBytesNb != 0) && (st25r95SPIRxCtx.protocol != ST25R95_PROTOCOL_ISO18092))
    {
        platformLog("[%s]", hex2Str(st25r95SPIRxCtx.BufCRC, 2));
    }
#endif /* ST25R95_DEBUG */

    if ((!st25r95SPIRxCtx.rmvCRC) && (st25r95SPIRxCtx.protocol == ST25R95_PROTOCOL_ISO18092) && (rcvdLen == len))
    {
        /* increase room for CRC*/
        st25r95SPIRxCtx.rxBuf[rcvdLen++] = 0x00;
        st25r95SPIRxCtx.rxBuf[rcvdLen++] = 0x00;
#if ST25R95_DEBUG
        platformLog("{%4.4X}", 0x0000);
#endif /* ST25R95_DEBUG */
        
    }
    
#if ST25R95_DEBUG
    platformLog(" %s", hex2Str(st25r95SPIRxCtx.additionalRespBytes, additionalRespBytesNb));
    platformLog(" (retCode=%d)\r\n", retCode);
#endif /* ST25R95_DEBUG */
    
    /* update *rxRcvdLen if not null pointer */
    if (st25r95SPIRxCtx.rxRcvdLen != NULL)
    {
        (*st25r95SPIRxCtx.rxRcvdLen) = rcvdLen;
    }
    #if RFAL_FEATURE_LISTEN_MODE
    if (st25r95SPIRxCtx.protocol == ST25R95_PROTOCOL_CE_ISO14443A)
    {
        st25r95SPIRxCtx.inListen = false;
        // we can assume Field is ON and AC State is Active because we just received some Data from the Reader
        st25r95SPIGetLmState(true); /* store lmState */
    }
    #endif /* RFAL_FEATURE_LISTEN_MODE */
    st25r95SPIRxCtx.retCode = retCode;
    return (retCode);
}

/*******************************************************************************/
ReturnCode st25r95SPIGetRxStatus(void)
{
    return (st25r95SPIRxCtx.retCode);
}

bool st25r95SPIIsTransmitCompleted(void)
{
    return (true);
}

bool st25r95SPIIsInListen(void)
{
    return (st25r95SPIRxCtx.inListen);
}

/*******************************************************************************/
void st25r95SPIIdle(uint8_t dacDataL, uint8_t dacDataH, uint8_t WUPeriod)
{
    Idle[ST25R95_IDLE_WUPERIOD_OFFSET] = WUPeriod;
    Idle[ST25R95_IDLE_DACDATAL_OFFSET] = dacDataL;
    Idle[ST25R95_IDLE_DACDATAH_OFFSET] = dacDataH;
    platformSpiSendCmd(Idle[0], Idle + 2, Idle[ST25R95_CMD_LENGTH_OFFSET], false);
    #if ST25R95_DEBUG
    platformLog("[%10d] >>>> %s\r\n", platformGetSysTick(), hex2Str(Idle, Idle[ST25R95_CMD_LENGTH_OFFSET] + 2));
    #endif /* ST25R95_DEBUG */
}

/*******************************************************************************/
void st25r95SPIGetIdleResponse(void)
{
    uint8_t respBuffer[ST25R95_IDLE_RESPONSE_BUFLEN];
    
    platformSpiRead(respBuffer, &respBuffer[2], ST25R95_IDLE_RESPONSE_BUFLEN - 2);
    #if ST25R95_DEBUG
    platformLog("[%10d] <<<< %s\r\n", platformGetSysTick(), hex2Str(respBuffer, respBuffer[ST25R95_CMD_LENGTH_OFFSET] + 2));
    #endif /* ST25R95_DEBUG */
 
}

/*******************************************************************************/
void st25r95SPIKillIdle(void)
{
    ReturnCode retCode = RFAL_ERR_NONE;
    
    st25r95SPI_nIRQ_IN_Pulse();
    /* Poll the ST25R95 until it is ready to transmit */
    retCode = st25r95SPIPollRead(ST25R95_CONTROL_POLL_TIMEOUT);
        
    if (retCode == RFAL_ERR_NONE)
    {
        st25r95SPIGetIdleResponse();
    }
    
}
#endif /* ST25R95_INTERFACE_SPI */
