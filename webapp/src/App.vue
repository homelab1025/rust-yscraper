<template>
  <div class="page">
    <header class="hero">
      <i class="pi pi-comments hero-icon" aria-hidden="true"></i>
      <h1>YScraper Dashboard</h1>
    </header>

    <main class="cards">
      <Card class="card">
        <template #title>
          Review Item
        </template>
        <template #subtitle>
          User X
        </template>
        <template #content>
          <p>
            This is the item preview. Use the actions below to keep or discard it.
          </p>
        </template>
        <template #footer>
          <div class="actions" role="group" aria-label="Card actions">
            <Button label="Keep" icon="pi pi-save" @click="onDiscard"/>
            <Button label="Discard" icon="pi pi-trash" severity="danger" @click="onDiscard"/>
          </div>
        </template>
      </Card>

    </main>
  </div>
</template>

<script setup>
import {ref} from 'vue';
import Card from 'primevue/card';
import Button from 'primevue/button';

const keepChecked = ref(false);

function onDiscard() {
  // Placeholder for discard logic
  // eslint-disable-next-line no-console
  console.log('Discard clicked', {keepChecked: keepChecked.value});
}
</script>

<style>
:root {
  --page-padding: clamp(16px, 4vw, 32px);
}

html, body, #app {
  height: 100%;
}

body {
  margin: 0;
}

.page {
  min-height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.hero {
  text-align: center;
  padding: var(--page-padding) var(--page-padding) 0;
}

.hero-icon {
  font-size: 2rem;
}

.subtitle {
  opacity: 0.8;
  margin-top: 0.25rem;
}

.cards {
  flex: 1;
  margin: auto; /* centers vertically and horizontally within remaining space */
  //display: flex;
  //flex-direction: column;
  //gap: 1rem;
  //align-items: center;
}

.card {
  width: 100%;
}

.actions {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
}

.keep {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
}

.card :where(ul, ol) {
  margin: 0.5rem 0 0 1.25rem;
}

.mr-2 {
  margin-right: 0.5rem;
}
</style>
