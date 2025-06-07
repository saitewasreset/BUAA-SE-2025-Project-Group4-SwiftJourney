<template>
  <div class="homepage">
    <!-- 主要内容区域 -->
    <div class="main-content">
      <!-- 顶部搜索区域 -->
      <div class="search-section">
        <div class="search-container">
          <!-- 火车票搜索卡片 -->
          <div class="search-card-wrapper">
            <TicketSearch />
          </div>
          
          <!-- 酒店搜索卡片 -->
          <div class="search-card-wrapper">
            <HotelSearchCard />
          </div>
        </div>
      </div>

      <!-- 旅行信息区域 -->
      <div class="travel-section">
        <div class="section-header">
          <h2 class="section-title">我的行程</h2>
          <p class="section-subtitle">您即将开始的精彩旅程</p>
        </div>
        <TravelInfo />
      </div>

      <!-- 快捷服务区域 -->
      <div class="services-section">
        <div class="section-header">
          <h2 class="section-title">快捷服务</h2>
          <p class="section-subtitle">一站式旅行服务</p>
        </div>
        
        <div class="services-grid">
          <div class="service-card" @click="goToTrainTicket">
            <div class="service-icon train-icon">
              <el-icon size="32"><Bicycle /></el-icon>
            </div>
            <div class="service-content">
              <h3 class="service-title">火车票</h3>
              <p class="service-desc">快速预订火车票</p>
            </div>
          </div>

          <div class="service-card" @click="goToHotel">
            <div class="service-icon hotel-icon">
              <el-icon size="32"><House /></el-icon>
            </div>
            <div class="service-content">
              <h3 class="service-title">酒店预订</h3>
              <p class="service-desc">舒适住宿选择</p>
            </div>
          </div>

          <div class="service-card" @click="goToMeal">
            <div class="service-icon meal-icon">
              <el-icon size="32"><Food /></el-icon>
            </div>
            <div class="service-content">
              <h3 class="service-title">火车餐</h3>
              <p class="service-desc">旅途美食体验</p>
            </div>
          </div>

          <div class="service-card">
            <div class="service-icon support-icon">
              <el-icon size="32"><Service /></el-icon>
            </div>
            <div class="service-content">
              <h3 class="service-title">客户服务</h3>
              <p class="service-desc">24小时在线支持</p>
            </div>
          </div>
        </div>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
// import { Bicycle, House, Food, Service, MapLocation, Camera, Mountain, OfficeBuilding } from '@element-plus/icons-vue';
import TicketSearch from '@/components/TicketSearch/TicketSearch.vue';
import TravelInfo from '@/components/TravelInfo/TravelInfo.vue';
import HotelSearchCard from '@/components/HotelSearch/HotelSearchCard.vue';

const router = useRouter();

// 快捷服务导航
function goToTrainTicket() {
  router.push('/trainTicket');
}

function goToHotel() {
  router.push('/hotel');
}

function goToMeal() {
  router.push('/meal');
}

const recommendations = ref([
  {
    id: 1,
    title: '北京·故宫博物院',
    description: '探索中华文明的瑰宝，感受皇家宫殿的壮丽',
    // icon: OfficeBuilding,
    imageClass: 'beijing-gradient',
    tag: '文化古迹',
    price: 299
  },
  {
    id: 2,
    title: '上海·外滩夜景',
    description: '璀璨霓虹下的都市风情，黄浦江畔的浪漫时光',
    // icon: Camera,
    imageClass: 'shanghai-gradient',
    tag: '都市风光',
    price: 199
  },
  {
    id: 3,
    title: '杭州·西湖美景',
    description: '人间天堂的诗意山水，江南水乡的温婉风情',
    // icon: Mountain,
    imageClass: 'hangzhou-gradient',
    tag: '自然风光',
    price: 159
  },
  {
    id: 4,
    title: '成都·宽窄巷子',
    description: '品味巴蜀文化的韵味，体验悠闲慢生活',
    // icon: MapLocation,
    imageClass: 'chengdu-gradient',
    tag: '民俗文化',
    price: 229
  }
]);
</script>

<style lang="scss" scoped>
.homepage {
  min-height: 100vh;
  background: linear-gradient(135deg, #f0f9ff 0%, #e0f2fe 25%, #fef3c7 75%, #fef7cd 100%);
  position: relative;
  
  &::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: 
      radial-gradient(circle at 20% 80%, rgba(120, 119, 198, 0.1) 0%, transparent 50%),
      radial-gradient(circle at 80% 20%, rgba(255, 119, 198, 0.1) 0%, transparent 50%),
      radial-gradient(circle at 40% 40%, rgba(120, 219, 226, 0.1) 0%, transparent 50%);
    pointer-events: none;
  }
}

.main-content {
  position: relative;
  z-index: 1;
  max-width: 1400px;
  margin: 0 auto;
  padding: 80px 20px 40px;
}

.search-section {
  margin-bottom: 60px;
}

.search-container {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 40px;
  max-width: 1200px;
  margin: 0 auto;
}

.search-card-wrapper {
  display: flex;
  justify-content: center;
}

.travel-section,
.services-section,
.recommendations-section {
  margin-bottom: 60px;
}

.section-header {
  text-align: center;
  margin-bottom: 40px;
}

.section-title {
  font-size: 2.5rem;
  font-weight: 800;
  margin: 0 0 8px 0;
  background: linear-gradient(135deg, #1e40af 0%, #059669 50%, #dc2626 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  letter-spacing: -0.02em;
}

.section-subtitle {
  font-size: 1.1rem;
  margin: 0;
  color: #64748b;
  font-weight: 500;
}

.services-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 24px;
  max-width: 1200px;
  margin: 0 auto;
}

.service-card {
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(20px);
  border-radius: 20px;
  padding: 32px;
  display: flex;
  align-items: center;
  gap: 20px;
  transition: all 0.3s cubic-bezier(0.23, 1, 0.32, 1);
  cursor: pointer;
  border: 1px solid rgba(255, 255, 255, 0.6);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08);

  &:hover {
    transform: translateY(-8px) scale(1.02);
    box-shadow: 0 20px 48px rgba(0, 0, 0, 0.12);
    background: rgba(255, 255, 255, 1);
  }
}

.service-icon {
  width: 64px;
  height: 64px;
  border-radius: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all 0.3s ease;

  &.train-icon {
    background: linear-gradient(135deg, #3b82f6, #1e40af);
    color: white;
    box-shadow: 0 4px 16px rgba(59, 130, 246, 0.4);
  }

  &.hotel-icon {
    background: linear-gradient(135deg, #10b981, #047857);
    color: white;
    box-shadow: 0 4px 16px rgba(16, 185, 129, 0.4);
  }

  &.meal-icon {
    background: linear-gradient(135deg, #f59e0b, #d97706);
    color: white;
    box-shadow: 0 4px 16px rgba(245, 158, 11, 0.4);
  }

  &.support-icon {
    background: linear-gradient(135deg, #8b5cf6, #7c3aed);
    color: white;
    box-shadow: 0 4px 16px rgba(139, 92, 246, 0.4);
  }
}

.service-content {
  flex: 1;
}

.service-title {
  font-size: 1.25rem;
  font-weight: 700;
  margin: 0 0 4px 0;
  color: #1f2937;
}

.service-desc {
  font-size: 0.95rem;
  color: #6b7280;
  margin: 0;
  line-height: 1.5;
}

.recommendations-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 28px;
  max-width: 1200px;
  margin: 0 auto;
}

.recommendation-card {
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(20px);
  border-radius: 20px;
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.23, 1, 0.32, 1);
  cursor: pointer;
  border: 1px solid rgba(255, 255, 255, 0.6);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08);

  &:hover {
    transform: translateY(-8px);
    box-shadow: 0 20px 48px rgba(0, 0, 0, 0.12);
  }
}

.recommendation-image {
  position: relative;
  height: 200px;
  overflow: hidden;
}

.recommendation-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 0.3s ease;

  &.beijing-gradient {
    background: linear-gradient(135deg, #dc2626, #991b1b);
  }

  &.shanghai-gradient {
    background: linear-gradient(135deg, #7c3aed, #5b21b6);
  }

  &.hangzhou-gradient {
    background: linear-gradient(135deg, #059669, #047857);
  }

  &.chengdu-gradient {
    background: linear-gradient(135deg, #ea580c, #c2410c);
  }
}

.recommendation-card:hover .recommendation-placeholder {
  transform: scale(1.1);
}

.recommendation-overlay {
  position: absolute;
  top: 16px;
  left: 16px;
}

.recommendation-tag {
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(10px);
  color: #1f2937;
  padding: 6px 12px;
  border-radius: 20px;
  font-size: 0.875rem;
  font-weight: 600;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.recommendation-content {
  padding: 24px;
}

.recommendation-title {
  font-size: 1.25rem;
  font-weight: 700;
  margin: 0 0 8px 0;
  color: #1f2937;
}

.recommendation-desc {
  font-size: 0.95rem;
  color: #6b7280;
  margin: 0 0 16px 0;
  line-height: 1.6;
}

.recommendation-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.recommendation-price {
  font-size: 1.1rem;
  font-weight: 700;
  color: #dc2626;
}

// 响应式设计
@media (max-width: 1024px) {
  .search-container {
    grid-template-columns: 1fr;
    gap: 30px;
  }

  .section-title {
    font-size: 2rem;
  }

  .services-grid,
  .recommendations-grid {
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 20px;
  }
}

@media (max-width: 768px) {
  .main-content {
    padding: 60px 16px 32px;
  }

  .section-title {
    font-size: 1.75rem;
  }

  .section-subtitle {
    font-size: 1rem;
  }

  .service-card {
    padding: 24px;
    flex-direction: column;
    text-align: center;
    gap: 16px;
  }

  .services-grid,
  .recommendations-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 640px) {
  .search-container {
    gap: 24px;
  }

  .section-header {
    margin-bottom: 32px;
  }

  .travel-section,
  .services-section,
  .recommendations-section {
    margin-bottom: 48px;
  }
}
</style>