import { createRouter, createWebHistory } from 'vue-router'
import ScreeningView from '@/views/ScreeningView.vue'
import ActiveView from '@/views/ActiveView.vue'
import DischargedView from '@/views/DischargedView.vue'
import PatientDetailView from '@/views/PatientDetailView.vue'
import ReportsView from '@/views/ReportsView.vue'
import SettingsView from '@/views/SettingsView.vue'
import AppointmentsView from '@/views/AppointmentsView.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      redirect: '/screening',
    },
    {
      path: '/screening',
      name: 'screening',
      component: ScreeningView,
      meta: { title: 'คัดกรองผู้ป่วย' },
    },
    {
      path: '/active',
      name: 'active',
      component: ActiveView,
      meta: { title: 'ผู้ป่วยในการรักษา' },
    },
    {
      path: '/discharged',
      name: 'discharged',
      component: DischargedView,
      meta: { title: 'ผู้ป่วยจำหน่ายแล้ว' },
    },
    {
      path: '/appointments',
      name: 'appointments',
      component: AppointmentsView,
      meta: { title: 'การนัดหมาย' },
    },
    {
      path: '/patient/:hn',
      name: 'patient-detail',
      component: PatientDetailView,
      props: true,
      meta: { title: 'รายละเอียดผู้ป่วย' },
    },
    {
      path: '/reports',
      name: 'reports',
      component: ReportsView,
      meta: { title: 'รายงาน' },
    },
    {
      path: '/settings',
      name: 'settings',
      component: SettingsView,
      meta: { title: 'ตั้งค่า' },
    },
  ],
})

export default router