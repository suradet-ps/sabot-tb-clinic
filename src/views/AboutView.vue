<script setup lang="ts">
import {
  Stethoscope,
  ShieldCheck,
  HardDrive,
  Info,
  CheckCircle,
  Lock,
  DatabaseZap,
  Network,
} from 'lucide-vue-next'

// Section 1: Feature list
const features = [
  'คัดกรองและลงทะเบียนผู้ป่วยจากระบบ HOSxP',
  'ติดตามความก้าวหน้าการรักษาและแจ้งเตือนอัตโนมัติ',
  'บันทึกผลติดตามรายเดือน น้ำหนัก ผลเสมหะ และอาการข้างเคียง',
] as const

// Section 2: Security points
interface SecurityPoint {
  icon: object
  title: string
  desc: string
}

const securityPoints: SecurityPoint[] = [
  {
    icon: Lock,
    title: 'อ่านอย่างเดียว (Read-Only)',
    desc: 'โปรแกรมเชื่อมต่อกับฐานข้อมูล HOSxP ในโหมดอ่านข้อมูลเท่านั้น ไม่สามารถเพิ่ม แก้ไข หรือลบข้อมูลใดๆ ในระบบ HOSxP ได้',
  },
  {
    icon: DatabaseZap,
    title: 'ไม่มีการเขียนข้อมูลกลับ (No Write-Back)',
    desc: 'ข้อมูลทั้งหมดที่บันทึกในโปรแกรมนี้ (การลงทะเบียน บันทึกติดตาม ผลการรักษา) จะถูกเก็บในฐานข้อมูลภายในเครื่อง (SQLite) ภายในเครื่องเท่านั้น ไม่ผ่านหรือเขียนกลับสู่ HOSxP',
  },
  {
    icon: Network,
    title: 'การเชื่อมต่อภายในเครือข่าย (Local Network Only)',
    desc: 'การเชื่อมต่อกับ HOSxP ทำผ่านเครือข่ายภายในโรงพยาบาลเท่านั้น ไม่มีการส่งข้อมูลออกสู่อินเทอร์เน็ต',
  },
]

// Section 4: Version / system info
interface InfoItem {
  label: string
  value: string
}

const infoItems: InfoItem[] = [
  { label: 'โปรแกรม',           value: 'TB Clinic Management System' },
  { label: 'เวอร์ชัน',          value: '0.1.3' },
  { label: 'พัฒนาโดย',          value: 'ทีมเภสัชกรรม โรงพยาบาลสระโบสถ์' },
  { label: 'แพลตฟอร์ม',         value: 'Tauri 2.5 + Vue 3.5' },
  { label: 'ฐานข้อมูลภายในเครื่อง', value: 'SQLite' },
  { label: 'ฐานข้อมูล HIS',     value: 'HOSxP MySQL (Read-Only)' },
]
</script>

<template>
  <div class="view-root">

    <!--Page header -->
    <div class="view-header">
      <h1>เกี่ยวกับโปรแกรม</h1>
      <p>TB Clinic Management System — โรงพยาบาลสระโบสถ์</p>
    </div>

    <div class="about-cards">

      <!-- Section 1 — โปรแกรมนี้คืออะไร-->
      <div class="about-card">

        <div class="card-header">
          <div class="card-icon-wrap card-icon-wrap--teal">
            <Stethoscope :size="20" stroke-width="2" />
          </div>
          <div class="card-header-text">
            <h2 class="card-title">โปรแกรมนี้คืออะไร</h2>
          </div>
        </div>

        <p class="card-body-text">
          โปรแกรมนี้พัฒนาขึ้นเพื่อช่วยบริหารจัดการคลินิกวัณโรค (TB Clinic) โรงพยาบาลสระโบสถ์
          โดยเฉพาะ ช่วยให้เภสัชกรสามารถติดตามผู้ป่วยวัณโรค วางแผนการรักษา
          บันทึกผลการตรวจติดตาม ได้อย่างมีประสิทธิภาพ
        </p>

        <ul class="feature-list">
          <li v-for="feat in features" :key="feat" class="feature-item">
            <CheckCircle :size="15" stroke-width="2.5" class="feature-check" />
            <span>{{ feat }}</span>
          </li>
        </ul>

      </div>

      <!-- Section 2 — ความปลอดภัยของฐานข้อมูล HOSxP -->
      <div class="about-card">

        <div class="card-header">
          <div class="card-icon-wrap card-icon-wrap--green">
            <ShieldCheck :size="20" stroke-width="2" />
          </div>
          <div class="card-header-text">
            <h2 class="card-title">ความปลอดภัยของฐานข้อมูล HOSxP</h2>
          </div>
        </div>

        <!-- Alt-bg callout banner -->
        <div class="security-banner">
          <ShieldCheck :size="18" stroke-width="2" class="banner-shield" />
          <span class="banner-text">ฐานข้อมูล HOSxP ปลอดภัย 100%</span>
          <span class="banner-badge">ยืนยัน</span>
        </div>

        <!-- 3 security points -->
        <div class="security-points">
          <div
            v-for="(point, idx) in securityPoints"
            :key="point.title"
            class="security-point"
            :class="{ 'security-point--last': idx === securityPoints.length - 1 }"
          >
            <div class="security-point-icon">
              <component :is="point.icon" :size="16" stroke-width="2" />
            </div>
            <div class="security-point-text">
              <p class="security-point-title">{{ point.title }}</p>
              <p class="security-point-desc">{{ point.desc }}</p>
            </div>
          </div>
        </div>

      </div>

      <!-- Section 3 — ฐานข้อมูลภายในเครื่อง -->
      <div class="about-card">

        <div class="card-header">
          <div class="card-icon-wrap card-icon-wrap--blue">
            <HardDrive :size="20" stroke-width="2" />
          </div>
          <div class="card-header-text">
            <h2 class="card-title">ฐานข้อมูลภายในเครื่อง</h2>
          </div>
        </div>

        <p class="card-body-text">
          ข้อมูลคลินิก TB ทั้งหมดที่บันทึกในโปรแกรม เก็บอยู่ในไฟล์ฐานข้อมูล SQLite
          ภายในเครื่องคอมพิวเตอร์ที่ติดตั้งโปรแกรม
        </p>

        <table class="data-table">
          <thead>
            <tr>
              <th class="data-th">แหล่งข้อมูล</th>
              <th class="data-th">วัตถุประสงค์</th>
              <th class="data-th data-th--right">สิทธิ์การเข้าถึง</th>
            </tr>
          </thead>
          <tbody>
            <tr class="data-row">
              <td class="data-td data-td--source">HOSxP MySQL</td>
              <td class="data-td">ข้อมูลผู้ป่วย ประวัติการจ่ายยา</td>
              <td class="data-td data-td--badge">
                <span class="access-badge access-badge--orange">Read Only</span>
              </td>
            </tr>
            <tr class="data-row data-row--last">
              <td class="data-td data-td--source">SQLite ภายในเครื่อง</td>
              <td class="data-td">การลงทะเบียน แผนการรักษา บันทึกติดตาม</td>
              <td class="data-td data-td--badge">
                <span class="access-badge access-badge--teal">Read/Write</span>
              </td>
            </tr>
          </tbody>
        </table>

        <p class="backup-note">
          <Info :size="13" stroke-width="2" class="backup-note-icon" />
          แนะนำให้ทำการสำรองข้อมูล SQLite สม่ำเสมอผ่านหน้าตั้งค่า
        </p>

      </div>

      <!-- Section 4 — เวอร์ชันและข้อมูลระบบ -->
      <div class="about-card">

        <div class="card-header">
          <div class="card-icon-wrap card-icon-wrap--muted">
            <Info :size="20" stroke-width="2" />
          </div>
          <div class="card-header-text">
            <h2 class="card-title">เวอร์ชันและข้อมูลระบบ</h2>
          </div>
        </div>

        <div class="info-grid">
          <div v-for="item in infoItems" :key="item.label" class="info-row">
            <span class="info-label">{{ item.label }}</span>
            <span class="info-value">{{ item.value }}</span>
          </div>
        </div>

      </div>

    </div><!-- /.about-cards -->
  </div><!-- /.view-root -->
</template>

<style scoped>
/* Page root */
.view-root {
  padding: 32px 32px 48px;
  max-width: 780px;
}

/* Page header */
.view-header {
  margin-bottom: 28px;
}

.view-header h1 {
  font-family: var(--font);
  font-size: 22px;
  font-weight: 700;
  letter-spacing: -0.3px;
  color: var(--color-text);
  margin: 0 0 4px;
}

.view-header p {
  font-family: var(--font);
  font-size: 14px;
  color: var(--color-text-secondary);
  margin: 0;
}

/* Cards container */
.about-cards {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

/* Individual card */
.about-card {
  background: var(--color-bg);
  border: var(--border);
  border-radius: var(--radius-card);
  box-shadow: var(--shadow-card);
  padding: 24px;
}

/* Card header row */
.card-header {
  display: flex;
  align-items: flex-start;
  gap: 14px;
  margin-bottom: 18px;
}

/* Colored icon wrapper */
.card-icon-wrap {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.card-icon-wrap--teal {
  background: rgba(42, 157, 153, 0.12);
  color: var(--color-teal);
}

.card-icon-wrap--green {
  background: rgba(26, 174, 57, 0.10);
  color: var(--color-green);
}

.card-icon-wrap--blue {
  background: rgba(0, 117, 222, 0.10);
  color: var(--color-blue);
}

.card-icon-wrap--muted {
  background: rgba(0, 0, 0, 0.05);
  color: var(--color-text-muted);
}

/* Vertically centre the title text inside the 40px icon row */
.card-header-text {
  display: flex;
  flex-direction: column;
  justify-content: center;
  min-height: 40px;
}

.card-title {
  font-family: var(--font);
  font-size: 15px;
  font-weight: 700;
  letter-spacing: -0.15px;
  color: var(--color-text);
  margin: 0;
}

/* Body paragraph */
.card-body-text {
  font-family: var(--font);
  font-size: 14px;
  line-height: 1.65;
  color: var(--color-text-secondary);
  margin: 0 0 18px;
}

/* Feature list (Section 1) */
.feature-list {
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin: 0;
  padding: 0;
}

.feature-item {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  font-family: var(--font);
  font-size: 13.5px;
  font-weight: 500;
  color: var(--color-text);
  line-height: 1.5;
}

.feature-check {
  flex-shrink: 0;
  color: var(--color-teal);
  margin-top: 1px;
}

/* Security callout banner (Section 2) */
.security-banner {
  display: flex;
  align-items: center;
  gap: 10px;
  background: var(--color-bg-alt);
  border: var(--border);
  border-radius: var(--radius-md);
  padding: 12px 16px;
  margin-bottom: 20px;
}

.banner-shield {
  flex-shrink: 0;
  color: var(--color-green);
}

.banner-text {
  flex: 1;
  font-family: var(--font);
  font-size: 13.5px;
  font-weight: 600;
  letter-spacing: -0.1px;
  color: var(--color-text);
}

.banner-badge {
  flex-shrink: 0;
  padding: 3px 10px;
  border-radius: var(--radius-pill);
  background: rgba(26, 174, 57, 0.12);
  color: var(--color-green);
  font-family: var(--font);
  font-size: 11.5px;
  font-weight: 700;
  letter-spacing: 0.1px;
}

/* Security points list (Section 2) */
.security-points {
  display: flex;
  flex-direction: column;
}

.security-point {
  display: flex;
  align-items: flex-start;
  gap: 14px;
  padding: 14px 0;
  border-bottom: var(--border);
}

.security-point:first-child {
  padding-top: 0;
}

.security-point--last {
  border-bottom: none;
  padding-bottom: 0;
}

.security-point-icon {
  width: 34px;
  height: 34px;
  flex-shrink: 0;
  border-radius: var(--radius-sm);
  background: rgba(26, 174, 57, 0.08);
  color: var(--color-green);
  display: flex;
  align-items: center;
  justify-content: center;
  margin-top: 1px;
}

.security-point-text {
  flex: 1;
  min-width: 0;
}

.security-point-title {
  font-family: var(--font);
  font-size: 13.5px;
  font-weight: 600;
  color: var(--color-text);
  margin: 0 0 4px;
  line-height: 1.4;
}

.security-point-desc {
  font-family: var(--font);
  font-size: 13px;
  color: var(--color-text-secondary);
  margin: 0;
  line-height: 1.6;
}

/* Data table (Section 3) */
.data-table {
  width: 100%;
  border-collapse: collapse;
  margin-bottom: 14px;
}

.data-th {
  font-family: var(--font);
  font-size: 11.5px;
  font-weight: 600;
  letter-spacing: 0.1px;
  color: var(--color-text-muted);
  text-transform: uppercase;
  text-align: left;
  padding: 0 12px 9px;
  border-bottom: var(--border);
}

.data-th:first-child {
  padding-left: 0;
}

.data-th--right {
  text-align: right;
}

.data-row td {
  border-bottom: var(--border);
}

.data-row--last td {
  border-bottom: none;
}

.data-td {
  font-family: var(--font);
  font-size: 13.5px;
  color: var(--color-text);
  padding: 11px 12px;
  vertical-align: middle;
  line-height: 1.4;
}

.data-td:first-child {
  padding-left: 0;
}

.data-td--source {
  font-weight: 600;
  white-space: nowrap;
}

.data-td--badge {
  text-align: right;
  white-space: nowrap;
}

/* Access right badges */
.access-badge {
  display: inline-flex;
  align-items: center;
  padding: 3px 9px;
  border-radius: var(--radius-pill);
  font-family: var(--font);
  font-size: 11.5px;
  font-weight: 700;
  letter-spacing: 0.05px;
  white-space: nowrap;
}

.access-badge--orange {
  background: rgba(221, 91, 0, 0.10);
  color: var(--color-orange);
}

.access-badge--teal {
  background: rgba(42, 157, 153, 0.10);
  color: var(--color-teal);
}

/* Backup hint note */
.backup-note {
  display: flex;
  align-items: flex-start;
  gap: 7px;
  font-family: var(--font);
  font-size: 12.5px;
  color: var(--color-text-muted);
  margin: 0;
  line-height: 1.5;
}

.backup-note-icon {
  flex-shrink: 0;
  margin-top: 1px;
}

/* Info grid (Section 4) */
.info-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0;
}

.info-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 12px 0;
  border-bottom: var(--border);
}

/* Remove bottom border on the last two items (bottom row of 2-col grid) */
.info-row:nth-last-child(-n+2) {
  border-bottom: none;
}

/* Right-column items: separate with a left border + left padding */
.info-row:nth-child(even) {
  padding-left: 20px;
  border-left: var(--border);
}

.info-label {
  font-family: var(--font);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.15px;
  color: var(--color-text-muted);
  text-transform: uppercase;
}

.info-value {
  font-family: var(--font);
  font-size: 13.5px;
  font-weight: 500;
  color: var(--color-text);
  line-height: 1.4;
}
</style>
