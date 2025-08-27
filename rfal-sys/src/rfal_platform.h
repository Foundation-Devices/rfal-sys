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
/*! \file
 *
 *  \author
 *
 *  \brief Platform specific definition layer
 *
 *  This should contain all platform and hardware specifics such as
 *  GPIO assignment, system resources, locks, IRQs, etc
 *
 *  Each distinct platform/system/board must provide this definitions
 *  for all SW layers to use
 *
 */

#ifndef RFAL_PLATFORM_H
#define RFAL_PLATFORM_H

/*
******************************************************************************
* INCLUDES
******************************************************************************
*/
#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>

/*
******************************************************************************
* FFI
******************************************************************************
*/

extern void ffi_gpio_set(uint32_t port, uint32_t pin, bool state);
extern bool ffi_gpio_get(uint32_t port, uint32_t pin);

extern void ffi_delay_ms(uint32_t delay);
extern uint32_t ffi_get_ticks_ms(void);

extern void ffi_handle_error(const char *file, int line);

extern uint32_t ffi_irq_out(void);
extern uint32_t ffi_irq_in(void);

extern void ffi_spi_deselect(void);
extern void ffi_spi_select(void);
extern void ffi_spi_tx_rx(const uint8_t* tx, uint8_t* rx, size_t len);

/*
******************************************************************************
* GLOBAL DEFINES
******************************************************************************
*/

#define ST25R95_N_IRQ_OUT_PIN        ffi_irq_out()         /*!< GPIO pin used for ST25R95 nIRQ_OUT            */
#define ST25R95_N_IRQ_OUT_PORT       ffi_irq_out()         /*!< GPIO port used for ST25R95 nIRQ_OUT           */
#define ST25R95_N_IRQ_IN_PIN         ffi_irq_in()          /*!< GPIO pin used for ST25R95 nIRQ_OIN            */
#define ST25R95_N_IRQ_IN_PORT        ffi_irq_in()          /*!< GPIO port used for ST25R95 nIRQ_OUT           */

#define ST25R95_TAGDETECT_DEF_CALIBRATION 0x7C             /*!< Tag Detection Calibration default value                    */
#define ST25R95_TAGDETECT_CALIBRATE       true             /*!< False: use default value, True: call calibration procedure */

/*
******************************************************************************
* GLOBAL MACROS
******************************************************************************
*/

#define platformGpioSet(port, pin)         ffi_gpio_set((port), (pin), true)     /*!< Turns the given GPIO High                   */
#define platformGpioClear(port, pin)       ffi_gpio_set((port), (pin), false)    /*!< Turns the given GPIO Low                    */
#define platformGpioToggle(port, pin)      ffi_gpio_set((port), (pin), !ffi_gpio_get((port), (pin))) /*!< Toggles the given GPIO  */
#define platformGpioIsHigh(port, pin)      (ffi_gpio_get((port), (pin)) == true) /*!< Checks if the given LED is High             */
#define platformGpioIsLow(port, pin)       (!platformGpioIsHigh((port), (pin)))  /*!< Checks if the given LED is Low              */

#define platformGetSysTick()               ffi_get_ticks_ms()                    /*!< Get System Tick ( 1 tick = 1 ms)            */
#define platformTimerCreate(t)             (platformGetSysTick()+(t))            /*!< Create a timer with the given time (ms)     */
#define platformTimerIsExpired(timer)      (platformGetSysTick()>=(timer))       /*!< Checks if the given timer is expired        */
#define platformDelay(t)                   ffi_delay_ms(t)                       /*!< Performs a delay for the given time (ms)    */

#define platformErrorHandle()              ffi_handle_error(__FILE__,__LINE__)   /*!< Global error handler or trap                */

#define platformSpiSelect()                ffi_spi_select()                      /*!< SPI SS\CS: Chip|Slave Select                */
#define platformSpiDeselect()              ffi_spi_deselect()                    /*!< SPI SS\CS: Chip|Slave Deselect              */
#define platformSpiTxRx(txBuf, rxBuf, len) ffi_spi_tx_rx(txBuf, rxBuf, len)      /*!< SPI transceive                              */

extern char* hex2Str(unsigned char * data, size_t dataLen);
extern int logString(const char* format, ...);
#define platformLog(...)                   logString(__VA_ARGS__)                /*!< Log  method                                 */

/*
******************************************************************************
* GLOBAL VARIABLES
******************************************************************************
*/

/*
******************************************************************************
* RFAL FEATURES CONFIGURATION
******************************************************************************
*/

#define RFAL_FEATURE_LISTEN_MODE               false      /*!< Enable/Disable RFAL support for Listen Mode                               */
#define RFAL_FEATURE_WAKEUP_MODE               true       /*!< Enable/Disable RFAL support for the Wake-Up mode                          */
#define RFAL_FEATURE_LOWPOWER_MODE             false      /*!< Enable/Disable RFAL support for the Low Power mode                        */
#define RFAL_FEATURE_NFCA                      true       /*!< Enable/Disable RFAL support for NFC-A (ISO14443A)                         */
#define RFAL_FEATURE_NFCB                      false      /*!< Enable/Disable RFAL support for NFC-B (ISO14443B)                         */
#define RFAL_FEATURE_NFCF                      false      /*!< Enable/Disable RFAL support for NFC-F (FeliCa)                            */
#define RFAL_FEATURE_NFCV                      false      /*!< Enable/Disable RFAL support for NFC-V (ISO15693)                          */
#define RFAL_FEATURE_T1T                       true       /*!< Enable/Disable RFAL support for T1T (Topaz)                               */
#define RFAL_FEATURE_T2T                       true       /*!< Enable/Disable RFAL support for T2T                                       */
#define RFAL_FEATURE_T4T                       false      /*!< Enable/Disable RFAL support for T4T                                       */
#define RFAL_FEATURE_ST25TB                    false      /*!< Enable/Disable RFAL support for ST25TB                                    */
#define RFAL_FEATURE_ST25xV                    false      /*!< Enable/Disable RFAL support for ST25TV/ST25DV                             */
#define RFAL_FEATURE_DYNAMIC_ANALOG_CONFIG     false      /*!< Enable/Disable Analog Configs to be dynamically updated (RAM)             */
#define RFAL_FEATURE_DPO                       false      /*!< Enable/Disable RFAL Dynamic Power Output support                          */
#define RFAL_FEATURE_ISO_DEP                   false      /*!< Enable/Disable RFAL support for ISO-DEP (ISO14443-4)                      */
#define RFAL_FEATURE_ISO_DEP_POLL              false      /*!< Enable/Disable RFAL support for Poller mode (PCD) ISO-DEP (ISO14443-4)    */
#define RFAL_FEATURE_ISO_DEP_LISTEN            false      /*!< Enable/Disable RFAL support for Listen mode (PICC) ISO-DEP (ISO14443-4)   */
#define RFAL_FEATURE_NFC_DEP                   false      /*!< Enable/Disable RFAL support for NFC-DEP (NFCIP1/P2P)                      */

#define RFAL_FEATURE_ISO_DEP_IBLOCK_MAX_LEN    256U       /*!< ISO-DEP I-Block max length. Please use values as defined by rfalIsoDepFSx */
#define RFAL_FEATURE_NFC_DEP_BLOCK_MAX_LEN     254U       /*!< NFC-DEP Block/Payload length. Allowed values: 64, 128, 192, 254           */
#define RFAL_FEATURE_NFC_RF_BUF_LEN            258U       /*!< RF buffer length used by RFAL NFC layer                                   */

#define RFAL_FEATURE_ISO_DEP_APDU_MAX_LEN      512U       /*!< ISO-DEP APDU max length. Please use multiples of I-Block max length       */
#define RFAL_FEATURE_NFC_DEP_PDU_MAX_LEN       512U       /*!< NFC-DEP PDU max length.                                                   */

/*
******************************************************************************
* RFAL FEATURES FALLBACK DEFAULT CONFIGURATION
******************************************************************************
*/

#include <rfal_defConfig.h>

#endif /* RFAL_PLATFORM_H */
