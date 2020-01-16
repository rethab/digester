<template>
  <div>
    <h1 class="display-1 d-flex justify-center">Settings</h1>
    <SetTimezone />
    <DeleteAccount />
  </div>
</template>

<script>
import SetTimezone from "@/components/settings/SetTimezone.vue";
import DeleteAccount from "@/components/settings/DeleteAccount.vue";
export default {
  components: {
    SetTimezone,
    DeleteAccount
  }
};
</script>