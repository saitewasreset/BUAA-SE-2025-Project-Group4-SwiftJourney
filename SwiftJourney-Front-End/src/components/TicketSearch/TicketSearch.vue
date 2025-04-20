<template>
    <!-- TODO: 设置日期可以选择的时间 -->
    <div class="TicketSearch">
        <el-card shadow="always">
            <div class="SelectTicketMode">
                <a-radio-group :value="selectedTicketMode" @change="handleSelectTicketMode" class="ModeRadio">
                    <a-radio value="OneWay">直达</a-radio>
                    <a-radio value="Transfer">中转</a-radio>
                </a-radio-group>
            </div>
            <div class="SelectContainer">
                <div class="SelectCity">
                    <CitySelect 
                    v-if="isCurChooseRefActive" 
                    :el="inputRef"
                    @handleCityClick="handleCityClick"
                    />
                    <div class="DepartureCity">
                        <p>出发城市</p>
                        <a-input
                            id="DepartureCityInput"
                            @Focus="handleInputFocus('DepartureCityInput')"
                            class="CityInput"
                            :bordered="false"
                            size="large"
                            v-model:value="departureCity"
                            placeholder="出发城市"
                            style="background-color: transparent; width: 100%;">
                        </a-input>
                    </div>
                    <div class="SwapButton">
                        <a-button
                            shape="circle"
                            :icon="h(SwapOutlined)"
                            @click="swapCitys"
                            style="border: none;">
                        </a-button>
                    </div>
                    <div class="ArrivalCity">
                        <p>到达城市</p>
                        <a-input
                            id="ArrivalCityInput"
                            @Focus="handleInputFocus('ArrivalCityInput')"
                            class="CityInput"
                            :bordered="false"
                            size="large"
                            v-model:value="arrivalCity"
                            placeholder="到达城市"
                            style="background-color: transparent; width: 100%; text-align: right;">
                        </a-input>
                    </div>
                </div>
                <div class="SelectDate">
                    <p>出发日期</p>
                    <a-date-picker
                        suffix-icon=""
                        id="DatePicker"
                        size="large"
                        :locale="locale"
                        :format="dateFormat"
                        :bordered="false"
                        />
                </div>
                <div class="SearchButton">
                    <a-button type="primary" size="large">
                        <template #icon>
                          <SearchOutlined />
                        </template>
                        查询
                      </a-button>
                </div>
            </div>
        </el-card>
    </div>
</template>

<script setup lang="ts">
    import { ref, h, nextTick } from 'vue';
    import { onMounted, onUnmounted } from 'vue';
    
    import { SwapOutlined, SearchOutlined } from '@ant-design/icons-vue';
    
    //-------------DatePicker----------------

    import locale from 'ant-design-vue/es/date-picker/locale/zh_CN';
    import dayjs from 'dayjs';
    import 'dayjs/locale/zh-cn';

    dayjs.locale('zh-cn');

    const dateFormat = 'YYYY-MM-DD     dddd';

    // --------------------------------------

    // ----------selectTicketMode------------

    const selectedTicketMode = ref('OneWay');

    function handleSelectTicketMode(e: Event) {
        if(e.target)
            selectedTicketMode.value = e.target.value;
    }

    // --------------------------------------

    // -------------SelectCity---------------

    const departureCity = ref<string>('');
    const arrivalCity = ref<string>('');

    function swapCitys() {
        const temp = departureCity.value;
        departureCity.value = arrivalCity.value;
        arrivalCity.value = temp;
    }

    const inputRef = ref<HTMLElement | undefined>(undefined)

    const isCurChooseRefActive = ref<boolean>(false);

    import CitySelect from './CitySelect/CitySelect.vue';

    const selectedInputId = ref<string>('');

    const isSelectorFocused = ref<boolean>(false);

    async function handleInputFocus(id: string) {
        selectedInputId.value = id;
        const inputElement = document.getElementById(id) as HTMLElement;
        inputRef.value = inputElement;
        isCurChooseRefActive.value = false;
        await nextTick();
        isCurChooseRefActive.value = true;
    }

    function handleCityClick(item: Object) {
        const cityName: string = item.cityName;
        if(selectedInputId.value === 'DepartureCityInput') {
            departureCity.value = cityName;
        } else if(selectedInputId.value === 'ArrivalCityInput') {
            arrivalCity.value = cityName;
        }
        isCurChooseRefActive.value = false;
    };


    function handleGlobalClick(event: MouseEvent) {
        const citySelectElement = document.querySelector('.city_choose_wrap');
        const inputElement = inputRef.value;

        if (
            citySelectElement &&
            !citySelectElement.contains(event.target as Node) &&
            inputElement &&
            !inputElement.contains(event.target as Node)
        ) {
            isCurChooseRefActive.value = false;
        }
    }

    onMounted(() => {
        document.addEventListener('click', handleGlobalClick);
    });

    onUnmounted(() => {
        document.removeEventListener('click', handleGlobalClick);
    });

    //------------------------------------------

</script>

<style lang="css" scoped>
.TicketSearch {
    height: 100%;
    display: flex;
    justify-content: center;
}
.el-card {
    height: 200px;
    border-radius: 15px;
}
.el-card__body {
    width: 100%;
    height: 100%;
    padding-left: 40px;
    padding-top: 40px; 
    padding-bottom: 40px;
}

.SelectTicketMode {
    display: flex;
    align-items: center;
    margin-bottom: 20px;
    border-radius: 8px;
}
.ModeRadio {
    .ant-radio-wrapper-checked {
        span {
            color: rgb(43,132,255);
        }
    }
    span {
        font-size: large;
        color: black;
    }
}

.SelectContainer {
    display: flex;
    height: 10%;
    width: 100%;
    align-items: center;
}

.SelectCity {
    background-color: rgb(248,248,248);
    font-size: small;
    display: flex;
    border-radius: 8px;
    padding: 20px;
    padding-top: 18px;
    padding-bottom: 12px;
    p {
        display: block;
        color: rgb(189,190,194);
        align-items: center;
        margin-left: 10px;
        margin-bottom: 0%;
    }
    .SwapButton {
        align-items: center;
        display: flex;
    }
    .CityInput {
        display: block;
        font-size: 1.25rem;
        font-weight: bolder;
    }
    .ArrivalCity {
        text-align: right;
        p {
            margin-left: 0%;
            margin-right: 10px;
        }
    }
}


.SelectDate {
    margin-left: 5%;
    background-color: rgb(248,248,248);
    display: block;
    border-radius: 8px;
    padding: 20px;
    padding-top: 18px;
    padding-bottom: 12px;
    height: 10%;
    p {
        display: block;
        font-size: small;
        color: rgb(189,190,194);
        align-items: center;
        margin-left: 10px;
        margin-bottom: 0%;
    }
}


.SelectDate .ant-picker {
    width: 100%;
    background-color: rgb(248,248,248);
    
}

::v-deep(#DatePicker) {
    font-size: 1.25rem;
    font-weight: bolder;
}

.SearchButton {
    display: grid;
    margin-right: 3%;
    margin-left: 3%;
    height: 90px;
    width: 100px;
    .ant-btn {
        height: 100%;
        width: 100%;
    }
}

</style>

<style lang="css">

.SearchButton .ant-btn span {
    font-size: 1.25rem;
    font-weight: bolder;
    margin-right: 10px;
}

</style>
  