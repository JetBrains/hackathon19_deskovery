//
// Created by Ilia.Motornyi on 26/08/2019.
//
#include "main.h"

extern TIM_HandleTypeDef htim1;
extern TIM_HandleTypeDef HMOTOR_TIM;
extern TIM_HandleTypeDef EL_TIM;
extern TIM_HandleTypeDef ER_TIM;
extern ADC_HandleTypeDef hadc1;
static volatile int64_t left_ticks_var = 0;
static volatile int64_t right_ticks_var = 0;

extern UART_HandleTypeDef huart2;
extern UART_HandleTypeDef huart3;

static inline void runMotorChannel(int pwr, GPIO_TypeDef *dirPort, uint16_t dirPin, uint16_t motorChannel) {
    if (pwr >= 0) {
        HAL_GPIO_WritePin(dirPort, dirPin, GPIO_PIN_RESET);
        __HAL_TIM_SET_COMPARE(&HMOTOR_TIM, motorChannel, pwr);
    } else {
        HAL_GPIO_WritePin(dirPort, dirPin, GPIO_PIN_SET);
        __HAL_TIM_SET_COMPARE(&HMOTOR_TIM, motorChannel, -pwr);
    }
    HAL_TIM_PWM_Start(&HMOTOR_TIM, motorChannel);
}

bool deskovery_motor(int pwrLeft, int pwrRight, bool recovery) {
//    if(! recovery && prxData.alarm) {//todo recovery
//        pwrLeft = 0;
//        pwrRight = 0;
//    }
    runMotorChannel(pwrLeft, ML_DIR_GPIO_Port, ML_DIR_Pin, ML_TIM_CH);
    runMotorChannel(pwrRight, MR_DIR_GPIO_Port, MR_DIR_Pin, MR_TIM_CH);
    return true;
}

VL53L1_Dev_t centerSensor = {
        .I2cDevAddr = 0x52,
        .new_data_ready_poll_duration_ms = 1000,
};

void VLO53L1A1_ResetPin(int state) {
    HAL_GPIO_WritePin(VL53_RST_GPIO_Port, VL53_RST_Pin, state ? GPIO_PIN_SET : GPIO_PIN_RESET);
}

void setupSensor(VL53L1_DEV dev) {
    uint16_t wordData;
    uint8_t byteData;
    __unused int status = 0;
    VLO53L1A1_ResetPin(0); // Shutdown center sensor
    HAL_Delay(2);
    VLO53L1A1_ResetPin(1); // run center sensor
    HAL_Delay(2);
    HAL_ADCEx_InjectedStart_IT(&hadc1);

/* Those basic I2C read functions can be used to check your own I2C functions */
    status += VL53L1_RdByte(dev, 0x010F, &byteData);
    printf("VL53L1X Model_ID: %X\n", byteData);
    status += VL53L1_RdByte(dev, 0x0110, &byteData);
    printf("VL53L1X Module_Type: %X\n", byteData);
    status += VL53L1_RdWord(dev, 0x010F, &wordData);
    printf("VL53L1X: %X\n", wordData);
    status += VL53L1_WaitDeviceBooted(dev);
    printf("Chip booted\n");

    /* This function must to be called to initialize the sensor with the default setting  */
    status += VL53L1_DataInit(dev);
    status += VL53L1_StaticInit(dev);
    /* Optional functions to be used to change the main ranging parameters according the application requirements to get the best ranging performances */
    status += VL53L1_SetPresetMode(dev, VL53L1_PRESETMODE_LITE_RANGING);
    status += VL53L1_SetDistanceMode(dev, VL53L1_DISTANCEMODE_SHORT);
    status += VL53L1_SetMeasurementTimingBudgetMicroSeconds(dev, 10000);
    status += VL53L1_SetInterMeasurementPeriodMilliSeconds(dev, 0);


//  status = VL53L1X_SetOffset(dev,20); /* offset compensation in mm */
//  status = VL53L1X_SetROI(dev, 16, 16); /* minimum ROI 4,4 */
//	status = VL53L1X_CalibrateOffset(dev, 140, &offset); /* may take few second to perform the offset cal*/
//	status = VL53L1X_CalibrateXtalk(dev, 1000, &xtalk); /* may take few second to perform the xtalk cal */
    status += VL53L1_StartMeasurement(dev);   /* This function has to be called to enable the ranging */

    if (status) Error_Handler();
}


void deskoveryInit(void) {
    HAL_TIM_Encoder_Start(&EL_TIM, TIM_CHANNEL_ALL);
    HAL_TIM_Encoder_Start(&ER_TIM, TIM_CHANNEL_ALL);
    setupSensor(&centerSensor);
    LCD5110_init();
    setupWifi();
}

void deskoveryReadEncoders() {
    left_ticks_var += (int16_t) __HAL_TIM_GET_COUNTER(&EL_TIM);
    __HAL_TIM_SET_COUNTER(&EL_TIM, 0);
    __HAL_TIM_SET_COMPARE(&EL_TIM, TIM_CHANNEL_1, 0);
    __HAL_TIM_SET_COMPARE(&EL_TIM, TIM_CHANNEL_2, 0);
    right_ticks_var += (int16_t) __HAL_TIM_GET_COUNTER(&ER_TIM);
    __HAL_TIM_SET_COUNTER(&ER_TIM, 0);
    __HAL_TIM_SET_COMPARE(&ER_TIM, TIM_CHANNEL_1, 0);
    __HAL_TIM_SET_COMPARE(&ER_TIM, TIM_CHANNEL_2, 0);
}

__unused int _write(int file, char *ptr, int len) {
    HAL_UART_Transmit(&huart2, ptr, len, 200);
}

static bool prxLedOn = false;
volatile PrxData prxData = {.alarmRatio10 = 30, .alarms = {false, false, false, false}, .alarm = false,
        .darkResponse = {0, 0, 0, 0}, .lightResponse ={0, 0, 0, 0}};

void HAL_ADCEx_InjectedConvCpltCallback(__unused ADC_HandleTypeDef *hadc) {
    if (prxLedOn) {
        prxData.lightResponse[0] = HAL_ADCEx_InjectedGetValue(&hadc1, ADC_INJECTED_RANK_1);
        prxData.lightResponse[1] = HAL_ADCEx_InjectedGetValue(&hadc1, ADC_INJECTED_RANK_2);
        prxData.lightResponse[2] = HAL_ADCEx_InjectedGetValue(&hadc1, ADC_INJECTED_RANK_3);
        prxData.lightResponse[3] = HAL_ADCEx_InjectedGetValue(&hadc1, ADC_INJECTED_RANK_4);
        prxData.alarms[0] = (prxData.darkResponse[0] * 10 / (prxData.lightResponse[0] + 1)) < prxData.alarmRatio10;
        prxData.alarms[1] = (prxData.darkResponse[1] * 10 / (prxData.lightResponse[1] + 1)) < prxData.alarmRatio10;
        prxData.alarms[2] = (prxData.darkResponse[2] * 10 / (prxData.lightResponse[2] + 1)) < prxData.alarmRatio10;
        prxData.alarms[3] = (prxData.darkResponse[3] * 10 / (prxData.lightResponse[3] + 1)) < prxData.alarmRatio10;
        prxData.alarm = prxData.alarms[0] || prxData.alarms[1] || prxData.alarms[2] || prxData.alarms[3];
    } else {
        prxData.darkResponse[0] = HAL_ADCEx_InjectedGetValue(&hadc1, ADC_INJECTED_RANK_1);
        prxData.darkResponse[1] = HAL_ADCEx_InjectedGetValue(&hadc1, ADC_INJECTED_RANK_2);
        prxData.darkResponse[2] = HAL_ADCEx_InjectedGetValue(&hadc1, ADC_INJECTED_RANK_3);
        prxData.darkResponse[3] = HAL_ADCEx_InjectedGetValue(&hadc1, ADC_INJECTED_RANK_4);
    }
    if (prxData.alarm) {
        HAL_TIM_PWM_Stop(&HMOTOR_TIM, TIM_CHANNEL_1);
        HAL_TIM_PWM_Stop(&HMOTOR_TIM, TIM_CHANNEL_2);
    } else {
        HAL_TIM_PWM_Start(&HMOTOR_TIM, TIM_CHANNEL_1);
        HAL_TIM_PWM_Start(&HMOTOR_TIM, TIM_CHANNEL_2);

    }

    prxLedOn = !prxLedOn;
    HAL_GPIO_WritePin(PRX_EN_GPIO_Port, PRX_EN_Pin, prxLedOn ? GPIO_PIN_SET : GPIO_PIN_RESET);
    HAL_ADCEx_InjectedStart_IT(&hadc1);
}

__unused void led_control(bool on) {
    HAL_GPIO_WritePin(LD2_GPIO_Port, LD2_Pin, on ? GPIO_PIN_SET : GPIO_PIN_RESET);
}

__unused void delay_ms(long ms) {
    uint32_t tickstart = HAL_GetTick();
    uint32_t wait = ms;

    /* Add a period to guaranty minimum wait */
    if (wait < HAL_MAX_DELAY) {
        wait += (uint32_t) (uwTickFreq);
    }

    while ((HAL_GetTick() - tickstart) < wait) {
        idle();
    }
}

__unused void display_bg_control(int brightness) {
    if (brightness < 0) {
        brightness = 0;
    } else if (brightness > 99) {
        brightness = 99;
    }
    __HAL_TIM_SET_COMPARE(&htim1, TIM_CHANNEL_3, 99 - brightness);
    HAL_TIM_PWM_Start(&htim1, TIM_CHANNEL_3);
}

__unused unsigned long system_ticks() {
    return HAL_GetTick();
}

long left_ticks() {
    long l;
    __disable_irq();
    l = left_ticks_var;
    __enable_irq();
    return l;
}

long right_ticks() {
    long r;
    __disable_irq();
    r = right_ticks_var;
    __enable_irq();
    return r;

}

static VL53L1_RangingMeasurementData_t rangeData;

static void runRadar() {
    __unused int status = 0;

    status += VL53L1_WaitMeasurementDataReady(&centerSensor);
    status += VL53L1_GetRangingMeasurementData(&centerSensor, &rangeData);
    VL53L1_ClearInterruptAndStartMeasurement(&centerSensor);
    if (status) {
        rangeData.RangeStatus = -101;
    }
}


__unused void debug_output(const unsigned char *p, unsigned int len) {
    HAL_UART_Transmit(&huart2, (uint8_t *) p, len, 1000);
}

void idle() {
    runRadar();
    unsigned char buf[4];
}

__unused int radar_range() {
    return rangeData.RangeStatus == 0 ? rangeData.RangeMilliMeter : -1;
}

__unused bool setRadarMatrix(unsigned char x1, unsigned char y1, unsigned char x2, unsigned char y2) {
    VL53L1_UserRoi_t roi;
    roi.TopLeftX = x1;
    roi.TopLeftX = x2;
    roi.TopLeftX = x2;
    roi.TopLeftX = y2;
    return VL53L1_SetUserROI(&centerSensor, &roi) == VL53L1_ERROR_NONE;
}
