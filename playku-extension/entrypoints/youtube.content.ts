export default defineContentScript({
  matches: ['*://*.youtube.com/*'],
  runAt: 'document_end',
  
  main() {
    function injectButton() {
      const currentUrl = window.location.href;
      const isWatchOrPlaylist = currentUrl.includes('/watch') || currentUrl.includes('/playlist');

      // Remove button if we navigated away from watch/playlist pages
      const existingContainer = document.getElementById('playku-container');
      if (!isWatchOrPlaylist) {
        if (existingContainer) existingContainer.remove();
        return;
      }

      // Avoid duplicate insertions
      if (existingContainer) return;

      const container = document.createElement('div');
      container.id = 'playku-container';
      let isStarting = false;

      // Inner elements: options menu + launch button (using div instead of button to bypass browser-specific padding/line-height quirks)
      container.innerHTML = `
        <div id="playku-menu-options" style="display: flex; align-items: center; opacity: 0; width: 0px; transition: opacity 0.2s ease, width 0.3s cubic-bezier(0.25, 1, 0.5, 1), padding-left 0.3s cubic-bezier(0.25, 1, 0.5, 1); pointer-events: none; overflow: hidden; box-sizing: border-box; padding-left: 0px; flex-shrink: 0;">
          <label id="playku-ontop-label" style="display: flex; align-items: center; gap: 4px; color: #ffffff; font-family: system-ui, -apple-system, sans-serif; font-size: 9px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.6px; cursor: pointer; user-select: none; white-space: nowrap;">
            <input type="checkbox" id="playku-ontop-checkbox" checked style="accent-color: #ffffff; cursor: pointer; width: 11px; height: 11px; margin: 0; background: transparent; border: 1px solid rgba(255, 255, 255, 0.4);" />
            <span>On Top</span>
          </label>
          <div style="width: 1px; height: 10px; background: rgba(255, 255, 255, 0.2); margin-left: 6px; margin-right: 6px;"></div>
          <label id="playku-audio-label" style="display: flex; align-items: center; gap: 4px; color: #ffffff; font-family: system-ui, -apple-system, sans-serif; font-size: 9px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.6px; cursor: pointer; user-select: none; white-space: nowrap;">
            <input type="checkbox" id="playku-audio-checkbox" style="accent-color: #ffffff; cursor: pointer; width: 11px; height: 11px; margin: 0; background: transparent; border: 1px solid rgba(255, 255, 255, 0.4);" />
            <span>Audio</span>
          </label>
          <div id="playku-divider" style="width: 1px; height: 12px; background: rgba(255, 255, 255, 0.25); margin-left: 8px; margin-right: 2px;"></div>
        </div>
        <div id="playku-launch-btn" style="width: 30px; height: 30px; border: none; background: transparent; color: #ffffff; cursor: pointer; display: flex; align-items: center; justify-content: center; padding: 0; flex-shrink: 0; transition: transform 0.2s cubic-bezier(0.25, 1, 0.5, 1); box-sizing: border-box;">
          <svg viewBox="0 0 24 24" fill="currentColor" width="11" height="11" style="display: block; margin-left: 0.5px;">
            <path d="M8 5v14l11-7z" />
          </svg>
        </div>
      `;

      // Parent container styling - Premium Glassmorphic / Dark Theme (Ultra-minimal 32px)
      container.style.cssText = `
        position: fixed;
        bottom: 30px;
        right: 30px;
        z-index: 2147483647;
        height: 32px;
        width: 32px;
        background: #111111;
        border: 1px solid rgba(255, 255, 255, 0.25);
        border-radius: 50px;
        display: flex;
        align-items: center;
        justify-content: center; /* Perfectly center the div button when collapsed */
        box-shadow: 0 4px 16px rgba(0,0,0,0.5);
        transition: all 0.3s cubic-bezier(0.25, 1, 0.5, 1);
        overflow: hidden;
        box-sizing: border-box;
      `;

      const optionsMenu = container.querySelector('#playku-menu-options') as HTMLDivElement;
      const launchBtn = container.querySelector('#playku-launch-btn') as HTMLDivElement;

      // Expand / Collapse hover animations
      container.onmouseenter = () => {
        if (isStarting) return;
        container.style.width = '162px';
        container.style.background = '#161616';
        container.style.borderColor = 'rgba(255, 255, 255, 0.4)';
        container.style.justifyContent = 'flex-end'; // Align to right when expanded
        optionsMenu.style.opacity = '1';
        optionsMenu.style.width = '122px';
        optionsMenu.style.paddingLeft = '10px';
        optionsMenu.style.pointerEvents = 'auto';
      };

      container.onmouseleave = () => {
        if (isStarting) return;
        container.style.width = '32px';
        container.style.background = '#111111';
        container.style.borderColor = 'rgba(255, 255, 255, 0.25)';
        container.style.justifyContent = 'center'; // Center perfectly again when collapsed
        optionsMenu.style.opacity = '0';
        optionsMenu.style.width = '0px';
        optionsMenu.style.paddingLeft = '0px';
        optionsMenu.style.pointerEvents = 'none';
      };

      // Launch button icon subtle scale hover effect
      launchBtn.onmouseenter = () => {
        if (isStarting) return;
        launchBtn.style.transform = 'scale(1.15)';
      };
      launchBtn.onmouseleave = () => {
        if (isStarting) return;
        launchBtn.style.transform = 'scale(1)';
      };

      // Play on click (with Timestamp Extraction!)
      launchBtn.addEventListener('click', () => {
        if (isStarting) return;

        const ontopCheckbox = container.querySelector('#playku-ontop-checkbox') as HTMLInputElement;
        const audioCheckbox = container.querySelector('#playku-audio-checkbox') as HTMLInputElement;
        const isChecked = ontopCheckbox ? ontopCheckbox.checked : true;
        const isAudioChecked = audioCheckbox ? audioCheckbox.checked : false;

        // Parse the current URL so we can easily manipulate its parameters
        const currentUrl = new URL(window.location.href);
        
        // Try to find the active HTML5 video element on the YouTube page
        const videoElement = document.querySelector('video');
        
        // If the video exists and has started playing, grab the time
        if (videoElement && videoElement.currentTime > 0) {
          // Extract the current time in seconds and round it down
          const timeInSeconds = Math.floor(videoElement.currentTime);
          
          // Add or update the '?t=X' parameter in the URL
          currentUrl.searchParams.set('t', `${timeInSeconds}s`);
        }

        // Apply modern query parameters directly to searchParams
        if (isChecked) {
          currentUrl.searchParams.set('playku_ontop', 'true');
        }
        if (isAudioChecked) {
          currentUrl.searchParams.set('playku_audio', 'true');
        }

        const finalUrlString = currentUrl.toString();

        isStarting = true;

        // Visual feedback inside options menu
        const originalHTML = optionsMenu.innerHTML;
        optionsMenu.innerHTML = `
          <span style="color: #ffffff; font-family: system-ui, -apple-system, sans-serif; font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.8px; white-space: nowrap;">Starting...</span>
        `;

        // Dim launch button during start duration
        launchBtn.style.opacity = '0.5';
        launchBtn.style.pointerEvents = 'none';

        // Trigger custom protocol with modern URL query parameters
        window.location.assign(`playku://${finalUrlString}`);

        // Reset state and restore controls after 2.5 seconds
        setTimeout(() => {
          isStarting = false;
          optionsMenu.innerHTML = originalHTML;
          launchBtn.style.opacity = '1';
          launchBtn.style.pointerEvents = 'auto';

          // Auto-collapse if mouse left the container during starting state
          if (!container.matches(':hover')) {
            container.style.width = '32px';
            container.style.background = '#111111';
            container.style.borderColor = 'rgba(255, 255, 255, 0.25)';
            container.style.justifyContent = 'center';
            optionsMenu.style.opacity = '0';
            optionsMenu.style.width = '0px';
            optionsMenu.style.paddingLeft = '0px';
            optionsMenu.style.pointerEvents = 'none';
          }
        }, 2500);
      });

      document.body.appendChild(container);
    }

    // Initial check when page loads
    injectButton();

    // YouTube fires 'yt-navigate-finish' whenever a soft SPA navigation completes
    window.addEventListener('yt-navigate-finish', injectButton);
  },
});
