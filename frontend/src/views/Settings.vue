<template>
  <div>
    <h1 class="display-1 d-flex justify-center">Settings</h1>
    <TimezoneSettings />
  </div>
</template>

<script>
import TimezoneSettings from "@/components/TimezoneSettings.vue";
export default {
  components: {
    TimezoneSettings
  }
};
</script>