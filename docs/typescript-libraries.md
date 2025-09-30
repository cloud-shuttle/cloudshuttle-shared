# TypeScript Shared Libraries Guide

This guide covers the TypeScript shared libraries available in CloudShuttle.

## Available Libraries

### 1. Components (`@cloudshuttle/components`)

Reusable React components across the platform.

**Available Components:**
- `Button` - Consistent button styling and behavior
- `Input` - Form input components
- `Modal` - Modal dialog components
- `Table` - Data table with sorting and pagination
- `Form` - Form components and utilities
- `Layout` - Layout and navigation components

**Usage:**
```tsx
import { Button, Input, Modal } from '@cloudshuttle/components';

function MyComponent() {
  return (
    <div>
      <Button onClick={handleClick}>Click me</Button>
      <Input placeholder="Enter text" />
      <Modal isOpen={isOpen} onClose={closeModal}>
        Modal content
      </Modal>
    </div>
  );
}
```

### 2. Hooks (`@cloudshuttle/hooks`)

Custom React hooks for common functionality.

**Available Hooks:**
- `useApi` - API call management with loading states
- `useAuth` - Authentication state management
- `useLocalStorage` - Local storage with reactivity
- `useDebounce` - Debounce values and callbacks
- `usePagination` - Pagination logic
- `useForm` - Form state management

**Usage:**
```tsx
import { useApi, useAuth, useForm } from '@cloudshuttle/hooks';

function MyComponent() {
  const { user, login, logout } = useAuth();
  const { data, loading, error } = useApi('/api/users');
  const { values, handleChange, handleSubmit } = useForm({
    initialValues: { name: '', email: '' },
    onSubmit: submitForm,
  });

  // Component implementation
}
```

### 3. Types (`@cloudshuttle/types`)

Shared TypeScript type definitions.

**Available Types:**
- `api.ts` - API response and request types
- `user.ts` - User-related type definitions
- `tenant.ts` - Tenant and organization types
- `common.ts` - Common utility types
- `forms.ts` - Form-related types

**Usage:**
```tsx
import type { User, ApiResponse, Tenant } from '@cloudshuttle/types';

interface Props {
  user: User;
  tenant: Tenant;
}

function UserProfile({ user, tenant }: Props): ApiResponse<User> {
  // Component implementation
}
```

### 4. Utils (`@cloudshuttle/utils`)

Utility functions and helpers.

**Available Utilities:**
- `formatters.ts` - Data formatting functions
- `validators.ts` - Input validation functions
- `date.ts` - Date manipulation utilities
- `string.ts` - String processing utilities
- `array.ts` - Array manipulation utilities
- `object.ts` - Object utilities

**Usage:**
```tsx
import { formatCurrency, validateEmail, formatDate } from '@cloudshuttle/utils';

const formattedPrice = formatCurrency(29.99);
const isValidEmail = validateEmail(email);
const displayDate = formatDate(new Date());
```

### 5. API (`@cloudshuttle/api`)

API client utilities and interceptors.

**Features:**
- HTTP client setup with interceptors
- Request/response transformation
- Error handling
- Authentication headers

**Usage:**
```tsx
import { apiClient, createAuthInterceptor } from '@cloudshuttle/api';

const client = apiClient.create({
  baseURL: '/api',
  interceptors: [createAuthInterceptor()],
});

const response = await client.get('/users');
```

### 6. Stores (`@cloudshuttle/stores`)

State management stores using Zustand.

**Available Stores:**
- `authStore` - Authentication state
- `userStore` - User data management
- `tenantStore` - Tenant/organization data
- `notificationStore` - Notification management

**Usage:**
```tsx
import { useAuthStore, useUserStore } from '@cloudshuttle/stores';

function MyComponent() {
  const { user, isAuthenticated } = useAuthStore();
  const { updateProfile } = useUserStore();

  // Component implementation
}
```

## Development

### Component Development

1. Create component in appropriate directory
2. Export from `src/index.ts`
3. Add TypeScript types
4. Add Storybook stories
5. Add unit tests

### Hook Development

1. Implement hook in `src/` directory
2. Export from `src/index.ts`
3. Add TypeScript types
4. Add unit tests with React Testing Library

### Type Definitions

1. Define types in appropriate file
2. Export from `src/index.ts`
3. Ensure type-only exports where appropriate

## Styling

Components use CSS-in-JS with styled-components for consistent theming.

## Testing

All components and hooks should have comprehensive unit tests using Jest and React Testing Library.

## TypeScript Configuration

Libraries use strict TypeScript configuration for maximum type safety.
