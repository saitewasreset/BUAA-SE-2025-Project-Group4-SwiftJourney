<template>
    <div class="city_choose_wrap" :style="{ top: pos.top + 'px', left: pos.left + 'px' }">
        <div class="choose_right">
            <el-tabs v-model="activeTag" class="demo-tabs">
                <el-tab-pane label="搜索结果" name="hotWelcomed">
                    <!-- 有搜索结果时显示 -->
                    <div v-if="updateSuggestions && updateSuggestions.length > 0" class="city_name" style="display: flex; flex-wrap: wrap; gap: 10px;">
                        <p class="suggestion" v-for="item in updateSuggestions" :key="item" @click="handleCityClick(item)">
                            {{ item }}
                        </p>
                    </div>
                    <!-- 有输入但无搜索结果时显示 -->
                    <div v-else-if="userInput.trim() !== ''" class="no-results">
                        <div class="no-results-icon">
                            <el-icon size="48"><Search /></el-icon>
                        </div>
                        <h3 class="no-results-title">暂无搜索结果</h3>
                        <p class="no-results-subtitle">
                            未找到包含 "<span class="search-keyword">{{ userInput }}</span>" 的{{ getSearchTypeText() }}
                        </p>
                        <p class="no-results-tip">请尝试使用其他关键词或检查拼写</p>
                    </div>
                    <!-- 无输入时显示提示 -->
                    <div v-else class="search-hint">
                        <div class="search-hint-icon">
                            <el-icon size="40"><Edit /></el-icon>
                        </div>
                        <h4 class="search-hint-title">请输入{{ getSearchTypeText() }}名称</h4>
                        <p class="search-hint-subtitle">支持中文名称或拼音搜索</p>
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
import { Search, Edit } from '@element-plus/icons-vue';

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

// 新增方法：获取搜索类型文本
function getSearchTypeText() {
    switch (props.type) {
        case 'city':
            return '城市';
        case 'station':
            return '车站';
        case 'both':
        default:
            return '城市或车站';
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
        return []; // 返回空数组而不是 undefined
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

/* 新增样式：无搜索结果 */
.no-results {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px 20px;
    text-align: center;
    height: 200px;
}

.no-results-icon {
    margin-bottom: 16px;
    color: #9ca3af;
}

.no-results-title {
    font-size: 18px;
    font-weight: 600;
    color: #374151;
    margin: 0 0 8px 0;
}

.no-results-subtitle {
    font-size: 14px;
    color: #6b7280;
    margin: 0 0 8px 0;
    line-height: 1.5;
}

.search-keyword {
    color: #3d6cfe;
    font-weight: 600;
}

.no-results-tip {
    font-size: 12px;
    color: #9ca3af;
    margin: 0;
    line-height: 1.4;
}

/* 新增样式：搜索提示 */
.search-hint {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px 20px;
    text-align: center;
    height: 200px;
}

.search-hint-icon {
    margin-bottom: 16px;
    color: #d1d5db;
}

.search-hint-title {
    font-size: 16px;
    font-weight: 600;
    color: #6b7280;
    margin: 0 0 8px 0;
}

.search-hint-subtitle {
    font-size: 14px;
    color: #9ca3af;
    margin: 0;
    line-height: 1.5;
}

/* 响应式设计 */
@media (max-width: 480px) {
    .city_choose_wrap {
        max-width: 320px;
    }
    
    .no-results,
    .search-hint {
        padding: 30px 15px;
        height: 180px;
    }
    
    .no-results-title {
        font-size: 16px;
    }
    
    .search-hint-title {
        font-size: 14px;
    }
    
    .no-results-subtitle,
    .search-hint-subtitle {
        font-size: 13px;
    }
}
</style>
