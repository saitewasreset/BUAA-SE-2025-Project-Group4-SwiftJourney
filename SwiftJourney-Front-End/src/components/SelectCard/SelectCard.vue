<template>
    <div class="city_choose_wrap" :style="{ top: pos.top + 'px', left: pos.left + 'px' }">
        <div class="choose_right">
            <el-tabs v-model="activeTag" class="demo-tabs">
                <el-tab-pane label="推荐" name="hotWelcomed">
                    <div class="city_name" style="display: flex; flex-wrap: wrap; gap: 10px;">
                        <p class="suggestion" v-for="item in updateSuggestions" :key="item" @click="handleCityClick(item)">
                            {{ item }}
                        </p>
                    </div>
                </el-tab-pane>
                <el-tab-pane label="ABCDEF" name="ABCDEF">
                    <div class="city_name_wrap">
                        <div class="city_name_box" v-for="c in 'ABCDEF'.split('')" :key="c">
                            <div class="font-semibold">{{ c }}</div>
                            <div class="city_name">
                                <p v-for="(item, index) in partTwoByCharacter(c)" :key="index" @click="handleCityClick(item.cityName)">
                                    {{ item.cityName }}
                                </p>
                            </div>
                        </div>
                    </div>
                </el-tab-pane>
                <el-tab-pane label="GHIJ" name="GHIJ">
                    <div class="city_name_wrap">
                        <div class="city_name_box" v-for="c in 'GHIJ'.split('')" :key="c">
                            <div class="font-semibold">{{ c }}</div>
                            <div class="city_name">
                                <p v-for="(item, index) in partTwoByCharacter(c)" :key="index" @click="handleCityClick(item.cityName)">
                                    {{ item.cityName }}
                                </p>
                            </div>
                        </div>
                    </div>
                </el-tab-pane>
                <el-tab-pane label="KLMN" name="KLMN">
                    <div class="city_name_wrap">
                        <div class="city_name_box" v-for="c in 'KLMN'.split('')" :key="c">
                            <div class="font-semibold">{{ c }}</div>
                            <div class="city_name">
                                <p v-for="(item, index) in partTwoByCharacter(c)" :key="index" @click="handleCityClick(item.cityName)">
                                    {{ item.cityName }}
                                </p>
                            </div>
                        </div>
                    </div>
                </el-tab-pane>
                <el-tab-pane label="PQRSTUVW" name="PQRSTUVW">
                    <div class="city_name_wrap">
                        <div class="city_name_box" v-for="c in 'PQRSTUVW'.split('')" :key="c">
                            <div class="font-semibold">{{ c }}</div>
                            <div class="city_name">
                                <p v-for="(item, index) in partTwoByCharacter(c)" :key="index" @click="handleCityClick(item.cityName)">
                                    {{ item.cityName }}
                                </p>
                            </div>
                        </div>
                    </div>
                </el-tab-pane>
                <el-tab-pane label="XYZ" name="XYZ">
                    <div class="city_name_wrap">
                        <div class="city_name_box" v-for="c in 'XYZ'.split('')" :key="c">
                            <div class="font-semibold">{{ c }}</div>
                            <div class="city_name">
                                <p v-for="(item, index) in partTwoByCharacter(c)" :key="index" @click="handleCityClick(item.cityName)">
                                    {{ item.cityName }}
                                </p>
                            </div>
                        </div>
                    </div>
                </el-tab-pane>
            </el-tabs>
        </div>
    </div>
</template>

<script setup lang="ts">
import { reactive, onMounted, ref, computed } from 'vue';
import { useGeneralStore } from '@/stores/general';

const generalStore = useGeneralStore();

const props = defineProps({
    input: {
        type: String,
        required: true
    },
    el: {
        type: HTMLElement,
        default: null,
    },
    type: {
        type: String, //city | station | both
        default: 'both',
    }
});

const pos = reactive({
    top: 0,
    left: 0,
})

const activeTag = ref('hotWelcomed')
const emit = defineEmits(['handleCityClick']);

function calcModalPosition() {
    if (props.el) {
        pos.top = props.el.getBoundingClientRect().height + props.el.offsetTop + 20
        pos.left = props.el.offsetLeft
    }
}

import { pinyin } from 'pinyin-pro'

const cityList = computed(() =>{ 
    if(props.type == 'city') {
        return generalStore.CityPinYinList;
    } else if (props.type == 'station') {
        return generalStore.StationPinYinList;
    } else {
        return generalStore.BothPinYinList;
    }
});

function handleCityClick(item: string) {
    emit('handleCityClick', item);
}

function partTwoByCharacter(c: string) {
    c = c.toUpperCase()
    let result: {cityName: string, pinYin: string}[] = [];
    cityList.value.forEach((value) => {
        if(value.pinYin.charAt(0).toUpperCase() === c) {
            result.push(value);
        }
    });
    return result;
}

const userInput = computed(() => props.input);

const updateSuggestions = computed(() => {
    const chineseChars: string[] = [];
    const otherChars: string[] = [];
    const stringChars: string[] = [];
    const suggestions: string[] = [];

    if(userInput.value.trim() == '') {
        return;
    }

    for (const char of userInput.value) {
        if (char >= '\u4e00' && char <= '\u9fff') { // 判断字符是否为汉字
            chineseChars.push(char);
            stringChars.push(pinyin(char, { toneType: 'none' }));
            stringChars.push(" ");
        } else if (char == '\'') {
            otherChars.push(' ');
            stringChars.push(' ');
        } else {
            otherChars.push(char);
            stringChars.push(char);
        }
    }

    const chineseString = chineseChars.join('');
    const otherString = otherChars.join('');
    const stringString = stringChars.join('');

    if(props.type == 'both') {
        suggestionsWithType(chineseString, otherString, stringString, suggestions, 'city');
        suggestionsWithType(chineseString, otherString, stringString, suggestions, 'station');
    } else {
        suggestionsWithType(chineseString, otherString, stringString, suggestions, props.type as 'city' | 'station');
    }

    return suggestions.slice(0, 20);
});

function suggestionsWithType(chineseString: string, otherString: string, stringString: string, suggestions: string[], type: 'city' | 'station') {
    if(type == 'city') {
        if(chineseString == '') {
            pinyinCmp(generalStore.PinYinList, otherString, suggestions, generalStore.PinYinMapCity);
        }
        else if(generalStore.CityMapPinYin[chineseString]) {
            let pinyins = generalStore.CityMapPinYin[chineseString];
            if(pinyins != null) {
                pinyinCmp(pinyins, stringString, suggestions, generalStore.PinYinMapCity);
            }
        } else {
            for(let i = 1; i <= chineseString.length; i++) {
                let tep = chineseString.substring(i-1, i);
                let pinyins = generalStore.CityMapPinYin[tep];
                if(pinyins != null) {
                    pinyinCmp(pinyins, stringString, suggestions, generalStore.PinYinMapCity);
                }
            }
        }
    } else {
        if(chineseString == '') {
            pinyinCmp(generalStore.PinYinListStation, otherString, suggestions, generalStore.PinYinMapStation);
        }
        else if(generalStore.StationMapPinYin[chineseString]) {
            let pinyins = generalStore.StationMapPinYin[chineseString];
            if(pinyins != null) {
                pinyinCmp(pinyins, stringString, suggestions, generalStore.PinYinMapStation);
            }
        } else {
            for(let i = 1; i <= chineseString.length; i++) {
                let tep = chineseString.substring(i-1, i);
                let pinyins = generalStore.StationMapPinYin[tep];
                if(pinyins != null) {
                    pinyinCmp(pinyins, stringString, suggestions, generalStore.PinYinMapStation);
                }
            }
        }
    }
}

function pinyinCmp(pinyins: string[], pinYin: string, suggestions: string[], pinYinMapCity: { [key: string]: string[] }) {
    pinyins.forEach((value) => {
        const templateParts = value.split(" ");
        const testParts = pinYin.split(" ");
        if (testParts.length <= templateParts.length) {
            // 遍历测试字符串的每个区域，检查是否是从模板字符串对应区域的开头开始的子串
            for (let i = 0; i < testParts.length; i++) {
                // 如果测试字符串的区域不是从模板字符串对应区域开头开始的子串，返回false
                if (!templateParts[i].startsWith(testParts[i])) {
                    return;
                }
            }
            const cities = pinYinMapCity[value];
            if(cities != null) {
                cities.forEach((tep) => {
                    suggestions.push(tep);
                })
            }
        }
    })
}

onMounted(() => {
    calcModalPosition();
    generalStore.init();
})
</script>

<style lang="css" scoped>
.city_choose_wrap {
    position: absolute;
    background: #fff;
    box-shadow: 0px 0px 12px #ccc;
    border-radius: 8px;
    overflow: hidden;
    display: flex;
    height: 300px;
    width: auto;
    max-width: 477px;
    z-index: 999;

    .choose_right {
        background: #fff;
        padding: 4px 12px;
        height: full;
        overflow-y: auto;

        p {
            cursor: pointer;

            &:hover {
                color: #3d6cfe;
            }
        }

        .city_name_wrap {
            height: 224px;
            overflow-y: auto;

            .city_name_box {
                display: flex;
                gap: 20px;

                .font-semibold {
                    font-weight: 600;
                }

                .city_name {
                    display: flex;
                    flex-wrap: wrap;
                    gap: 8px;
                    font-size: 14px;
                    line-height: 20px;
                    color: #374151;
                    margin-bottom: 16px;
                }
            }
        }
    }

    .slide_active {
        background: #3d6cfe;
        color: #fff;
    }
}

.suggestion {
    font-size: 16px;
    color: #374151;
    margin-bottom: 0;
    cursor: pointer;

    &:hover {
        color: #3d6cfe;
    }
}
</style>
