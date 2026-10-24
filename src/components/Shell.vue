<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { RouterLink, RouterView, useRouter } from 'vue-router'

const router = useRouter()

// §6.4：数字键切页面（输入框聚焦时不抢键）
const NAV = [
  { key: '1', path: '/today', label: '☀️ 今日' },
  { key: '2', path: '/tasks', label: '✅ 任务' },
  { key: '3', path: '/trajectory', label: '🧭 轨迹' },
  { key: '4', path: '/projects', label: '📁 课题' },
  { key: '5', path: '/ddl', label: '⏰ DDL' },
  { key: '6', path: '/papers', label: '📄 论文' },
  { key: '7', path: '/weekly', label: '🗓 周报' },
  { key: '8', path: '/review', label: '📈 回顾' },
  { key: '9', path: '/settings', label: '⚙ 设置' },
]

function onKeydown(e: KeyboardEvent) {
  const target = e.target as HTMLElement
  if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return
  const item = NAV.find((n) => n.key === e.key)
  if (item) router.push(item.path)
}

onMounted(() => window.addEventListener('keydown', onKeydown))
onUnmounted(() => window.removeEventListener('keydown', onKeydown))
</script>

<template>
  <div class="shell">
    <aside class="side">
      <div class="brand">科迹 <span class="sub">SciTrail</span></div>
      <nav>
        <RouterLink v-for="n in NAV" :key="n.path" :to="n.path" class="nav-item">
          <span>{{ n.label }}</span>
          <span class="kbd">{{ n.key }}</span>
        </RouterLink>
      </nav>
      <div class="side-foot">Ctrl+Alt+K 速录 · 托盘常驻</div>
    </aside>
    <main class="main">
      <RouterView />
    </main>
  </div>
</template>

<style scoped>
.shell {
  display: flex;
  height: 100vh;
}

.side {
  width: var(--sidebar-w);
  flex: none;
  border-right: 1px solid var(--line);
  background: var(--card);
  padding: 14px 10px;
  display: flex;
  flex-direction: column;
}

.brand {
  font-weight: 600;
  padding: 4px 10px 14px;
}

.brand .sub {
  color: var(--muted-fg);
  font-weight: 400;
  font-size: 12px;
}

nav {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.nav-item {
  display: flex;
  align-items: center;
  padding: 7px 10px;
  border-radius: 8px;
  color: var(--fg);
  text-decoration: none;
}

.nav-item:hover {
  background: var(--hover);
}

.nav-item.router-link-active {
  background: var(--hover);
  font-weight: 500;
}

.nav-item .kbd {
  margin-left: auto;
  color: var(--muted-fg);
  font-size: 11px;
}

.side-foot {
  margin-top: auto;
  padding: 8px 10px 2px;
  color: var(--muted-fg);
  font-size: 11px;
}

.main {
  flex: 1;
  overflow-y: auto;
}
</style>
