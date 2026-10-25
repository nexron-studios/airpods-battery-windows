<template>
  <div class="relative h-full aspect-[1/2.05] transform-3d">
    <!-- Back plate pushed behind the front, so the phone shows its thickness when it tilts -->
    <div class="absolute inset-0 rounded-[22%/11%] bg-device-frame-shade -translate-z-0.5" />
    <div class="absolute inset-0 rounded-[22%/11%] bg-linear-to-br from-device-frame to-device-frame-shade p-[1.5px]">
      <div class="relative size-full overflow-hidden rounded-[18%/9%] bg-device-screen">
        <div
          class="absolute -inset-1 bg-radial-[at_30%_25%] from-device-wallpaper-glow via-device-wallpaper to-device-screen"
        />
        <div class="absolute top-[6%] left-1/2 h-[6%] w-[36%] -translate-x-1/2 rounded-full bg-device-screen" />
        <div
          class="absolute inset-y-0 w-1/2 bg-linear-to-r from-transparent via-device-sheen to-transparent animate-device-sheen"
        />
      </div>
    </div>
  </div>
</template>
