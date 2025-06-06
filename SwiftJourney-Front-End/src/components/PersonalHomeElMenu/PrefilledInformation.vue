<template>
  <div class="prefilled-container">
    <div class="prefilled-card">
      <!-- 头部标题区域 -->
      <div class="card-header">
        <div class="header-content">
          <h1 class="page-title">预填信息</h1>
          <p class="page-subtitle">管理常用乘车人信息，让出行更便捷</p>
        </div>
        <div class="header-actions">
          <el-button 
            type="primary" 
            class="add-passenger-btn"
            @click="showAddPassengerDialog"
            :icon="Plus"
          >
            添加乘车人
          </el-button>
        </div>
      </div>

      <!-- 分割线 -->
      <div class="divider"></div>

      <!-- 乘车人列表 -->
      <div class="passengers-content">
        <div v-if="allPersonalInfos.length === 0" class="empty-state">
          <div class="empty-icon">
            <el-icon size="64"><User /></el-icon>
          </div>
          <h3 class="empty-title">暂无乘车人信息</h3>
          <p class="empty-subtitle">添加常用乘车人，让购票更快捷</p>
          <el-button 
            type="primary" 
            class="empty-add-btn"
            @click="showAddPassengerDialog"
            :icon="Plus"
          >
            添加第一个乘车人
          </el-button>
        </div>

        <div v-else class="passengers-grid">
          <div 
            v-for="(personalInfo, index) in allPersonalInfos" 
            :key="personalInfo.personalId"
            class="passenger-card"
            :class="{ 'is-self': personalInfo.default }"
          >
            <div class="passenger-header">
              <div class="passenger-info">
                <div class="name-section">
                  <h3 class="passenger-name">{{ personalInfo.name }}</h3>
                  <span v-if="personalInfo.default" class="self-badge">本人</span>
                </div>
                <span class="passenger-type adult">
                  成人
                </span>
              </div>
              <div class="passenger-actions">
                <!-- 本人信息只显示编辑按钮，其他乘车人显示编辑和删除按钮 -->
                <el-button 
                  class="action-btn edit-btn"
                  :icon="Edit"
                  size="small"
                  text
                  @click="editPersonalInfo(personalInfo)"
                >
                  编辑
                </el-button>
                <el-button 
                  v-if="!personalInfo.default"
                  class="action-btn delete-btn"
                  :icon="Delete"
                  size="small"
                  text
                  @click="confirmDeletePersonalInfo(personalInfo)"
                >
                  删除
                </el-button>
              </div>
            </div>

            <div class="passenger-details">
              <div class="detail-item">
                <span class="detail-label">身份证号</span>
                <span class="detail-value">{{ desensitizeIdCard(personalInfo.identityCardId) }}</span>
              </div>
              <div class="detail-item">
                <span class="detail-label">偏好座位</span>
                <div class="seat-preference-visual">
                  <div class="seat-row">
                    <div 
                      v-for="seat in ['A', 'B', 'C', 'D', 'F']" 
                      :key="seat"
                      class="seat"
                      :class="{ 
                        'selected': personalInfo.preferredSeatLocation === seat,
                        'window': seat === 'A' || seat === 'F',
                        'aisle': seat === 'C'
                      }"
                    >
                      {{ seat }}
                    </div>
                  </div>
                  <div class="seat-legend">
                    {{ getSeatLocationText(personalInfo.preferredSeatLocation) }}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 添加/编辑乘车人对话框 -->
    <el-dialog
      v-model="dialogVisible"
      :title="dialogTitle"
      width="600px"
      class="passenger-dialog"
      :close-on-click-modal="false"
    >
      <div class="dialog-form">
        <div class="form-grid">
          <!-- 如果是编辑本人信息，显示提示 -->
          <div v-if="isEditingSelf" class="form-item full-width">
            <div class="self-edit-notice">
              <el-icon class="notice-icon"><InfoFilled /></el-icon>
              <span class="notice-text">您正在编辑本人信息，修改后将同步更新到个人资料</span>
            </div>
          </div>

          <div class="form-item">
            <div class="item-header">
              <label class="form-label">姓名 <span class="required">*</span></label>
              <span v-if="isEdit" class="readonly-tag">只读</span>
              <span v-else class="readonly-placeholder"></span>
            </div>
            <el-input
              v-model="formData.name"
              placeholder="请输入真实姓名"
              maxlength="20"
              :disabled="isEdit"
              clearable
            />
          </div>

          <div class="form-item">
            <div class="item-header">
              <label class="form-label">身份证号 <span class="required">*</span></label>
              <span v-if="isEdit" class="readonly-tag">只读</span>
              <span v-else class="readonly-placeholder"></span>
            </div>
            <el-input
              v-model="formData.identityCardId"
              placeholder="请输入18位身份证号"
              maxlength="18"
              :disabled="isEdit"
              :class="{ 'error-input': identityCardIdError && !isEdit }"
              clearable
              @input="checkIdentityCardId"
              @change="checkIdentityCardId"
            />
            <div v-if="identityCardIdError && !isEdit" class="error-message">
              {{ identityCardIdErrorMsg }}
            </div>
          </div>

          <div class="form-item full-width">
            <div class="item-header">
              <label class="form-label">偏好座位</label>
              <span class="readonly-placeholder"></span>
            </div>
            <div class="seat-selection">
              <div class="seat-row">
                <div 
                  v-for="seat in ['A', 'B', 'C']" 
                  :key="seat"
                  class="seat clickable"
                  :class="{ 
                    'selected': formData.preferredSeatLocation === seat,
                    'window': seat === 'A',
                    'aisle': seat === 'C'
                  }"
                  @click="formData.preferredSeatLocation = seat"
                >
                  {{ seat }}
                </div>
                <!-- 过道间距 -->
                <div class="aisle-gap"></div>
                <div 
                  v-for="seat in ['D', 'F']" 
                  :key="seat"
                  class="seat clickable"
                  :class="{ 
                    'selected': formData.preferredSeatLocation === seat,
                    'window': seat === 'F',
                    'aisle': seat === 'D'
                  }"
                  @click="formData.preferredSeatLocation = seat"
                >
                  {{ seat }}
                </div>
              </div>
              <div class="seat-description">
                <div class="desc-item">
                  <span class="seat-type window-type">窗</span>
                  <span class="desc-text">靠窗 (A, F)</span>
                </div>
                <div class="desc-item">
                  <span class="seat-type aisle-type">道</span>
                  <span class="desc-text">靠过道 (C, D)</span>
                </div>
                <div class="desc-item">
                  <span class="seat-type middle-type">中</span>
                  <span class="desc-text">中间 (B)</span>
                </div>
              </div>
              <div v-if="formData.preferredSeatLocation" class="selected-info">
                <el-icon class="check-icon"><Check /></el-icon>
                <span class="selected-text">已选择：{{ getSeatLocationText(formData.preferredSeatLocation) }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <template #footer>
        <div class="dialog-footer">
          <el-button class="cancel-btn" @click="dialogVisible = false">
            取消
          </el-button>
          <el-button class="save-btn" type="primary" @click="savePersonalInfo">
            {{ getDialogButtonText() }}
          </el-button>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script lang="ts">
import { ElMessage, ElMessageBox } from 'element-plus';
import { Plus, User, Edit, Delete, InfoFilled, Check } from '@element-plus/icons-vue';
import { useUserStore } from '@/stores/user';
import { userApi } from '@/api/UserApi/userApi';
import type { UserApiResponseData } from '@/interface/userInterface';

type ResponseData = PersonalInfo[];

interface PersonalInfo {
  // 该用户身份的 UUID
  personalId: string;
  // 姓名
  name: string;
  // 身份证号
  identityCardId: string;
  // 偏好座位位置
  preferredSeatLocation?: "A" | "B" | "C" | "D" | "F";
  // 是否为默认个人资料，即，当前用户的身份
  default: boolean;
}

interface UpdatePersonalInfo {
  // 姓名
  name?: string;
  // 身份证号
  identityCardId: string;
  // 偏好座位位置
  preferredSeatLocation?: "A" | "B" | "C" | "D" | "F";
  // 是否为默认个人资料，即，当前用户的身份
  default?: boolean;
}

interface PersonalInfoForm {
  name: string;
  identityCardId: string;
  preferredSeatLocation: "A" | "B" | "C" | "D" | "F";
}

export default {
  components: {
    Plus,
    User,
    Edit,
    Delete,
    InfoFilled,
    Check,
  },
  data() {
    return {
      personalInfos: [] as PersonalInfo[],
      dialogVisible: false,
      isEdit: false,
      isEditingSelf: false,
      editingPersonalId: null as string | null,
      formData: {
        name: '',
        identityCardId: '',
        preferredSeatLocation: 'A', // 默认为A座
      } as PersonalInfoForm,
      // 身份证号验证状态
      identityCardIdError: false,
      identityCardIdErrorMsg: '',
      // Make icons available in template
      Plus,
      User,
      Edit,
      Delete,
      InfoFilled,
      Check,
    };
  },
  computed: {
    userStore() {
      return useUserStore();
    },
    dialogTitle() {
      if (this.isEditingSelf) {
        return '编辑默认信息';
      }
      return this.isEdit ? '编辑乘车人' : '添加乘车人';
    },
    allPersonalInfos() {
      // 查找并构造本人信息
      const selfPersonalInfo = this.personalInfos.find(p => p.default) || {
        personalId: '0',
        name: this.userStore.name,
        identityCardId: this.userStore.identityCardId,
        preferredSeatLocation: 'A',
        default: true
      };
      return [selfPersonalInfo, ...this.personalInfos.filter(p => !p.default)];
    },
  },
  mounted() {
    this.loadPersonalInfos();
  },
  methods: {
    // 加载乘车人列表
    async loadPersonalInfos() {
      const response: ResponseData = (await userApi.queryPersonalInfo()).data.data;
      this.personalInfos = response;
    },

    // 显示添加乘车人对话框
    showAddPassengerDialog() {
      this.isEdit = false;
      this.isEditingSelf = false;
      this.editingPersonalId = null;
      this.resetFormData();
      this.dialogVisible = true;
    },

    // 编辑乘车人
    editPersonalInfo(personalInfo: PersonalInfo) {
      this.isEdit = true;
      this.isEditingSelf = personalInfo.default;
      this.editingPersonalId = personalInfo.personalId;
      
      this.formData = {
        name: personalInfo.name,
        identityCardId: personalInfo.identityCardId,
        preferredSeatLocation: personalInfo.preferredSeatLocation || 'A', // 确保有默认值
      };
      this.dialogVisible = true;
    },

    // 确认删除乘车人
    confirmDeletePersonalInfo(personalInfo: PersonalInfo) {
      ElMessageBox.confirm(
        `确定要删除乘车人"${personalInfo.name}"吗？`,
        '删除确认',
        {
          confirmButtonText: '确定',
          cancelButtonText: '取消',
          type: 'warning',
        }
      ).then(() => {
        this.deletePersonalInfo(personalInfo);
      });
    },

    // 删除乘车人
    async deletePersonalInfo(personalInfo: PersonalInfo) {
      try {
        const res: UserApiResponseData = (await userApi.setPersonalInfo({identityCardId: personalInfo.identityCardId})).data;
        if(res.code === 200) {
          ElMessage.success('删除成功');
        } else {
          ElMessage.error('删除失败');
        }
      } catch (error) {
        ElMessage.error('删除失败');
      }
      await this.loadPersonalInfos();
    },

    // 保存乘车人
    async savePersonalInfo() {
      if (!this.validateForm()) {
        return;
      }

      try {
        if (this.isEdit) {
          const newPersonalInfo: UpdatePersonalInfo = {
            ...this.formData,
            default: this.isEditingSelf ? true: false,
          };
          const res: UserApiResponseData = (await userApi.setPersonalInfo(newPersonalInfo)).data;
          if(res.code === 200)
            ElMessage.success('修改成功');
          else
            ElMessage.error('修改失败');
        } else {
          const newPersonalInfo: UpdatePersonalInfo = {
            ...this.formData,
            default: false,
          };
          const res: UserApiResponseData = (await userApi.setPersonalInfo(newPersonalInfo)).data;
          if(res.code === 200)
            ElMessage.success('添加成功');
          else
            ElMessage.error('添加失败');
        }
        this.dialogVisible = false;
      } catch (error) {
        ElMessage.error(this.getErrorMessage());
      }
      await this.loadPersonalInfos();
    },

    // 身份证号检测（仿照注册页面逻辑）
    checkIdentityCardId() {
      this.identityCardIdError = false;
      this.identityCardIdErrorMsg = '';

      if (this.formData.identityCardId === '') {
        return;
      }

      const weight = [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2];
      const checkCode = ['1', '0', 'X', '9', '8', '7', '6', '5', '4', '3', '2'];

      if (this.formData.identityCardId.length !== 18) {
        this.identityCardIdError = true;
        this.identityCardIdErrorMsg = '身份证号码长度应为18位';
        return;
      }

      let sum = 0;
      for (let i = 0; i < 17; i++) {
        if (!/\d/.test(this.formData.identityCardId[i])) {
          this.identityCardIdError = true;
          this.identityCardIdErrorMsg = '身份证号码前17位应全部为数字';
          return;
        }
        sum += parseInt(this.formData.identityCardId[i], 10) * weight[i];
      }

      // 计算模11后的余数
      const mod = sum % 11;

      // 对比最后一位校验码
      const expectedCheckCode = checkCode[mod].toUpperCase();
      const actualCheckCode = this.formData.identityCardId[17].toUpperCase();

      if (actualCheckCode !== expectedCheckCode) {
        this.identityCardIdError = true;
        this.identityCardIdErrorMsg = '身份证号码校验失败';
      }
    },

    // 表单验证
    validateForm(): boolean {
      if (!this.formData.name.trim()) {
        ElMessage.error('请输入姓名');
        return false;
      }

      if (!this.formData.identityCardId.trim()) {
        ElMessage.error('请输入身份证号');
        return false;
      }

      // 执行身份证号检测
      this.checkIdentityCardId();
      if (this.identityCardIdError) {
        ElMessage.error(this.identityCardIdErrorMsg);
        return false;
      }

      return true;
    },

    // 重置表单数据
    resetFormData() {
      this.formData = {
        name: '',
        identityCardId: '',
        preferredSeatLocation: 'A', // 默认为A座
      };
      // 重置验证状态
      this.identityCardIdError = false;
      this.identityCardIdErrorMsg = '';
    },

    // 获取对话框按钮文本
    getDialogButtonText(): string {
      if (this.isEditingSelf) {
        return '保存本人信息';
      }
      return this.isEdit ? '保存修改' : '添加乘车人';
    },

    // 获取错误信息
    getErrorMessage(): string {
      if (this.isEditingSelf) {
        return '本人信息修改失败';
      }
      return this.isEdit ? '修改失败' : '添加失败';
    },

    // 脱敏身份证号
    desensitizeIdCard(idCard: string): string {
      if (!idCard || idCard.length < 8) return '未设置';
      return `${idCard.substring(0, 6)}********${idCard.substring(idCard.length - 2)}`;
    },

    // 获取座位位置描述文本
    getSeatLocationText(seatLocation: string | undefined): string {
      const seatMap = {
        A: 'A座 (靠窗)',
        B: 'B座 (中间)',
        C: 'C座 (靠过道)',
        D: 'D座 (靠过道)',
        F: 'F座 (靠窗)',
      };
      return seatMap[seatLocation as keyof typeof seatMap] || 'A座 (靠窗)'; // 默认返回A座
    },
  },
};
</script>

<style scoped>
/* 容器样式 */
.prefilled-container {
  min-height: 50vh;
  padding: 20px;
  display: flex;
  justify-content: center;
}

/* 主卡片 */
.prefilled-card {
  width: 100%;
  max-width: 1200px;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(20px);
  border-radius: 24px;
  box-shadow: 
    0 20px 40px rgba(0, 0, 0, 0.1),
    0 0 0 1px rgba(255, 255, 255, 0.2);
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.prefilled-card:hover {
  transform: translateY(-4px);
  box-shadow: 
    0 32px 64px rgba(0, 0, 0, 0.15),
    0 0 0 1px rgba(255, 255, 255, 0.3);
}

/* 卡片头部 */
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 40px 40px 20px;
  background: linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%);
}

.header-content {
  flex: 1;
}

.page-title {
  font-size: 32px;
  font-weight: 700;
  color: #1a202c;
  margin: 0 0 8px 0;
  letter-spacing: -0.5px;
}

.page-subtitle {
  font-size: 16px;
  color: #64748b;
  margin: 0;
  font-weight: 400;
}

.header-actions {
  margin-left: 24px;
}

.add-passenger-btn {
  padding: 12px 24px;
  border-radius: 12px;
  font-weight: 600;
  font-size: 15px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.add-passenger-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(102, 126, 234, 0.4);
}

/* 分割线 */
.divider {
  height: 1px;
  background: linear-gradient(90deg, transparent, #e2e8f0, transparent);
  margin: 0 40px;
}

/* 乘车人内容区域 */
.passengers-content {
  padding: 40px;
}

/* 空状态 */
.empty-state {
  text-align: center;
  padding: 60px 20px;
}

.empty-icon {
  color: #d1d5db;
  margin-bottom: 24px;
}

.empty-title {
  font-size: 24px;
  font-weight: 600;
  color: #374151;
  margin: 0 0 12px 0;
}

.empty-subtitle {
  font-size: 16px;
  color: #6b7280;
  margin: 0 0 32px 0;
}

.empty-add-btn {
  padding: 12px 32px;
  border-radius: 12px;
  font-weight: 600;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
}

.empty-add-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(102, 126, 234, 0.4);
}

/* 乘车人网格 */
.passengers-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
  gap: 24px;
}

/* 乘车人卡片 */
.passenger-card {
  background: #fff;
  border-radius: 16px;
  border: 2px solid #f1f5f9;
  padding: 24px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  overflow: hidden;
}

.passenger-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 4px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  transform: scaleX(0);
  transform-origin: left;
  transition: transform 0.3s;
}

.passenger-card:hover {
  border-color: #e2e8f0;
  transform: translateY(-4px);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.1);
}

.passenger-card:hover::before {
  transform: scaleX(1);
}

.passenger-card.is-self {
  border-color: #667eea;
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.05) 0%, rgba(118, 75, 162, 0.05) 100%);
}

.passenger-card.is-self::before {
  transform: scaleX(1);
}

.name-section {
  display: flex;
  align-items: center;
  gap: 8px;
}

.self-badge {
  display: inline-block;
  padding: 2px 8px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  font-size: 10px;
  font-weight: 600;
  border-radius: 12px;
  letter-spacing: 0.5px;
}

.passenger-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 20px;
}

.passenger-info {
  flex: 1;
}

.passenger-name {
  font-size: 18px;
  font-weight: 600;
  color: #1a202c;
  margin: 0 0 8px 0;
}

.passenger-type {
  display: inline-block;
  padding: 4px 12px;
  border-radius: 8px;
  font-size: 12px;
  font-weight: 500;
}

.passenger-type.adult {
  background: #dbeafe;
  color: #1e40af;
}

.passenger-type.child {
  background: #fef3c7;
  color: #d97706;
}

.passenger-type.student {
  background: #d1fae5;
  color: #059669;
}

.passenger-type.senior {
  background: #fce7f3;
  color: #be185d;
}

.passenger-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  padding: 6px 12px;
  border-radius: 8px;
  font-size: 12px;
  font-weight: 500;
  transition: all 0.2s;
}

.edit-btn {
  color: #059669;
}

.edit-btn:hover {
  background: #d1fae5;
  color: #047857;
}

.delete-btn {
  color: #dc2626;
}

.delete-btn:hover {
  background: #fee2e2;
  color: #b91c1c;
}

.passenger-details {
  display: grid;
  gap: 12px;
}

.detail-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid #f1f5f9;
}

.detail-item:last-child {
  border-bottom: none;
}

.detail-label {
  font-size: 14px;
  color: #6b7280;
  font-weight: 500;
}

.detail-value {
  font-size: 14px;
  color: #374151;
  font-weight: 500;
}

.seat-preference {
  padding: 2px 8px;
  background: #f3f4f6;
  border-radius: 6px;
  font-size: 12px;
}

/* 对话框样式 */
.passenger-dialog :deep(.el-dialog) {
  border-radius: 16px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
}

.passenger-dialog :deep(.el-dialog__header) {
  padding: 24px 24px 0;
  border-bottom: 1px solid #f1f5f9;
}

.passenger-dialog :deep(.el-dialog__title) {
  font-size: 20px;
  font-weight: 600;
  color: #1a202c;
}

.passenger-dialog :deep(.el-dialog__body) {
  padding: 24px;
}

.dialog-form {
  margin-bottom: 20px;
}

.form-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 20px;
}

.form-item {
  display: flex;
  flex-direction: column;
  min-height: 70px; /* 设置表单项最小高度 */
}

.form-item.full-width {
  grid-column: 1 / -1;
}

.form-label {
  font-size: 14px;
  font-weight: 600;
  color: #374151;
  margin: 0; /* 移除默认margin */
  line-height: 20px; /* 设置固定行高 */
}

.required {
  color: #dc2626;
}

.full-width {
  width: 100%;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 20px 24px 24px;
  border-top: 1px solid #f1f5f9;
}

.cancel-btn,
.save-btn {
  padding: 10px 20px;
  border-radius: 8px;
  font-weight: 500;
}

.cancel-btn {
  background: #f3f4f6;
  color: #6b7280;
  border: 1px solid #e5e7eb;
}

.save-btn {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
}

/* 座位选择样式 */
.seat-selection {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 20px;
  background: #f8fafc;
  border-radius: 12px;
  border: 1px solid #e2e8f0;
}

.seat-row {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  position: relative;
}

.seat.clickable {
  width: 48px;
  height: 48px;
  border: 2px solid #e2e8f0;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  font-weight: 600;
  color: #6b7280;
  background: #fff;
  cursor: pointer;
  transition: all 0.2s;
}

.seat.clickable:hover {
  transform: scale(1.05);
  border-color: #667eea;
  box-shadow: 0 2px 8px rgba(102, 126, 234, 0.2);
}

.seat.window {
  border-color: #3b82f6;
  color: #3b82f6;
}

.seat.aisle {
  border-color: #10b981;
  color: #10b981;
}

.seat.selected {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-color: #667eea;
  color: white;
  transform: scale(1.1);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
}

/* 过道间距 */
.aisle-gap {
  width: 24px;
  height: 2px;
  background: #d1d5db;
  margin: 0 8px;
  border-radius: 1px;
  position: relative;
}

.aisle-gap::before {
  content: '过道';
  position: absolute;
  top: -20px;
  left: 50%;
  transform: translateX(-50%);
  font-size: 10px;
  color: #9ca3af;
  white-space: nowrap;
}

/* 座位说明 */
.seat-description {
  display: flex;
  justify-content: center;
  gap: 24px;
  padding: 12px 0;
  border-top: 1px solid #e2e8f0;
}

.desc-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.seat-type {
  width: 20px;
  height: 20px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: 600;
}

.window-type {
  background: #dbeafe;
  color: #1e40af;
  border: 1px solid #3b82f6;
}

.aisle-type {
  background: #d1fae5;
  color: #047857;
  border: 1px solid #10b981;
}

.middle-type {
  background: #f3f4f6;
  color: #6b7280;
  border: 1px solid #d1d5db;
}

.desc-text {
  font-size: 12px;
  color: #6b7280;
  font-weight: 500;
}

/* 选中信息 */
.selected-info {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 8px 16px;
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.1) 0%, rgba(118, 75, 162, 0.1) 100%);
  border: 1px solid rgba(102, 126, 234, 0.2);
  border-radius: 8px;
}

.check-icon {
  color: #10b981;
  font-size: 16px;
}

.selected-text {
  font-size: 14px;
  color: #374151;
  font-weight: 500;
}

/* 座位偏好可视化样式（用于展示页面） */
.seat-preference-visual {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 8px;
}

.seat-preference-visual .seat-row {
  gap: 4px;
}

.seat-preference-visual .seat {
  width: 28px;
  height: 28px;
  border: 2px solid #e2e8f0;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
  color: #6b7280;
  background: #fff;
  transition: all 0.2s;
}

.seat-preference-visual .seat.window {
  border-color: #3b82f6;
  color: #3b82f6;
}

.seat-preference-visual .seat.aisle {
  border-color: #10b981;
  color: #10b981;
}

.seat-preference-visual .seat.selected {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-color: #667eea;
  color: white;
  transform: scale(1.1);
  box-shadow: 0 2px 8px rgba(102, 126, 234, 0.3);
}

.seat-legend {
  font-size: 12px;
  color: #6b7280;
  font-weight: 500;
  text-align: center;
}

/* 座位排列说明 */
.seat-row::before {
  content: '窗';
  font-size: 10px;
  color: #9ca3af;
  margin-right: 4px;
}

.seat-row::after {
  content: '窗';
  font-size: 10px;
  color: #9ca3af;
  margin-left: 4px;
}

/* 过道标识 */
.seat-row .seat:nth-child(3)::after {
  content: '';
  position: absolute;
  right: -12px;
  top: 50%;
  transform: translateY(-50%);
  width: 8px;
  height: 2px;
  background: #d1d5db;
  border-radius: 1px;
}

/* 错误输入框样式 */
.error-input :deep(.el-input__wrapper) {
  border-color: #f56565 !important;
  box-shadow: 0 0 0 2px rgba(245, 101, 101, 0.2) !important;
}

.error-input:hover :deep(.el-input__wrapper) {
  border-color: #f56565 !important;
}

/* 错误信息样式 */
.error-message {
  color: #f56565;
  font-size: 12px;
  margin-top: 4px;
  margin-left: 4px;
  font-weight: 500;
  line-height: 1.4;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .prefilled-container {
    padding: 16px;
  }

  .card-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 20px;
    padding: 24px 24px 16px;
  }

  .passengers-content {
    padding: 24px;
  }

  .passengers-grid {
    grid-template-columns: 1fr;
  }

  .form-grid {
    grid-template-columns: 1fr;
  }

  .dialog-footer {
    flex-direction: column-reverse;
  }

  .cancel-btn,
  .save-btn {
    width: 100%;
  }
}

/* 动画效果 */
@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.passenger-card {
  animation: fadeInUp 0.6s cubic-bezier(0.4, 0, 0.2, 1) forwards;
}

.passenger-card:nth-child(1) { animation-delay: 0.1s; }
.passenger-card:nth-child(2) { animation-delay: 0.2s; }
.passenger-card:nth-child(3) { animation-delay: 0.3s; }
.passenger-card:nth-child(4) { animation-delay: 0.4s; }

/* 座位预览点击功能 */
.seat.clickable {
  cursor: pointer;
  transition: all 0.2s;
}

.seat.clickable:hover {
  transform: scale(1.05);
  border-color: #667eea;
}

/* 禁用输入框样式 */
.form-item .el-input.is-disabled :deep(.el-input__wrapper) {
  background-color: #f8fafc;
  border-color: #e2e8f0;
  opacity: 0.8;
}

.form-item .el-input.is-disabled :deep(.el-input__inner) {
  color: #6b7280;
  cursor: not-allowed;
}

/* 本人信息编辑提示样式 */
.self-edit-notice {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.1) 0%, rgba(118, 75, 162, 0.1) 100%);
  border: 1px solid rgba(102, 126, 234, 0.2);
  border-radius: 8px;
  margin-bottom: 8px;
}

.notice-icon {
  color: #667eea;
  font-size: 16px;
}

.notice-text {
  font-size: 14px;
  color: #4a5568;
  font-weight: 500;
}

/* 表单项头部样式 */
.item-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
  min-height: 20px; /* 设置固定最小高度 */
}

/* 只读标签样式 */
.readonly-tag {
  font-size: 12px;
  color: #6b7280;
  background: #f3f4f6;
  padding: 4px 8px;
  border-radius: 8px;
  font-weight: 500;
  height: 20px; /* 固定高度 */
  display: flex;
  align-items: center;
}

/* 占位元素，保持高度一致 */
.readonly-placeholder {
  height: 20px; /* 与只读标签相同高度 */
  width: 1px;
  visibility: hidden;
}

/* 表单项统一样式 */
.form-item {
  display: flex;
  flex-direction: column;
  min-height: 70px; /* 设置表单项最小高度 */
}

.form-item .form-label {
  font-size: 14px;
  font-weight: 600;
  color: #374151;
  margin: 0; /* 移除默认margin */
  line-height: 20px; /* 设置固定行高 */
}

/* 输入框容器统一高度 */
.form-item .el-input,
.form-item .el-select {
  height: 40px; /* 统一输入框高度 */
}

.form-item .el-input :deep(.el-input__wrapper),
.form-item .el-select :deep(.el-input__wrapper) {
  height: 40px;
}
</style>