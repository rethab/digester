<template>
  <v-row cols="12">
    <v-col cols="5">
      <v-select v-model="value.frequency" :items="frequencies" append-icon></v-select>
    </v-col>
    <v-col cols="3">
      <v-select v-model="value.day" :disabled="!isWeekly" :items="days" append-icon></v-select>
    </v-col>
    <v-col cols="4">
      <v-select v-model="value.hour" :items="hours" append-icon></v-select>
    </v-col>
  </v-row>
</template>

<script>
export default {
  props: {
    value: {
      type: Object,
      required: true
    }
  },
  watch: {
    value() {
      this.$emit("input", this.value);
    }
  },
  data() {
    return {
      frequencies: ["Weekly", "Daily"],
      days: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
      hours: []
    };
  },
  computed: {
    isWeekly() {
      return this.value.frequency === "Weekly";
    }
  },
  mounted() {
    var hours = [];
    for (var h = 1; h < 25; h++) {
      var formatted = h < 10 ? "0" + h : "" + h;
      formatted = formatted + ":00";
      hours.push(formatted);
    }
    this.hours = hours;
  }
};
</script>