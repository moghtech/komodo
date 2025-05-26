# Testing Maintenance Windows Functionality

## ✅ Setup Complete
- Frontend running at: http://localhost:5173
- Backend running at: http://localhost:9120  
- Services: Core, Periphery, FerretDB all running

## 🧪 Test Steps

### 1. Basic Access Test
- [x] Frontend loads without TypeScript errors
- [x] All components compile successfully
- [x] Backend services are running and healthy

### 2. UI Navigation Test
1. Open frontend at http://localhost:5173
2. Login with default credentials (if needed)
3. Navigate to servers list
4. Select a server or create one
5. Verify "Maintenance" tab appears alongside Config, Stats, Docker, Resources

### 3. Maintenance Tab Test
1. Click on "Maintenance" tab
2. Should see:
   - Page title: "Maintenance Windows"
   - Description text about suppressing alerts
   - Empty state or existing maintenance windows
   - "Add Maintenance Window" button (if no existing windows)

### 4. Create Maintenance Window Test
1. Click "Add Maintenance Window" or "+" button
2. Fill out form:
   - Name: "Daily Backup"
   - Schedule Type: "Daily"
   - Start Time: 05:00
   - Duration: 30 minutes
   - Description: "Daily backup maintenance"
   - Enabled: true
3. Save the maintenance window
4. Verify it appears in the list

### 5. Edit/Delete Test
1. Click edit icon on created maintenance window
2. Modify duration to 60 minutes
3. Save changes
4. Verify changes are reflected
5. Test delete functionality

### 6. Different Schedule Types Test
1. Create Weekly maintenance window (e.g., Sunday 02:00)
2. Create OneTime maintenance window (specific date)
3. Verify all schedule types work correctly

### 7. Save/Reset Functionality Test
1. Make changes to maintenance windows
2. Verify "Save Changes" and "Reset" buttons appear
3. Test Reset functionality (should revert changes)
4. Test Save functionality (should persist changes)

### 8. Backend Integration Test
1. Check that maintenance windows are saved to server config
2. Verify data persists after page refresh
3. Test that alerts would be suppressed during active windows (if possible)

## 🐛 Issues Found
(Document any issues discovered during testing)

## ✅ Verification Checklist
- [ ] Maintenance tab appears in server interface
- [ ] MaintenanceWindows component renders correctly
- [ ] Can create new maintenance windows
- [ ] Can edit existing maintenance windows  
- [ ] Can delete maintenance windows
- [ ] All schedule types work (Daily, Weekly, OneTime)
- [ ] Save/Reset functionality works
- [ ] Data persists correctly
- [ ] No TypeScript errors
- [ ] No console errors in browser
- [ ] Responsive design works on different screen sizes