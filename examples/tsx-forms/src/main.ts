/**
 * Cliffy TSX Forms Example
 *
 * Demonstrates:
 * - Form state management with Behaviors
 * - Real-time validation with derived state
 * - Password strength indicator
 * - Character count limits
 * - Form submission handling
 * - Conditional UI based on validation state
 */

import init, { behavior, combine } from '@industrialalgebra/cliffy-core';
import { html, mount } from '@industrialalgebra/cliffy-core/html';

// Helper to combine 3-4 behaviors (replaces wedge)
function combine3<A, B, C, R>(
  a: ReturnType<typeof behavior<A>>,
  b: ReturnType<typeof behavior<B>>,
  c: ReturnType<typeof behavior<C>>,
  fn: (a: A, b: B, c: C) => R
): ReturnType<typeof behavior<R>> {
  return combine(combine(a, b, (x: A, y: B) => [x, y] as [A, B]), c, (pair: [A, B], z: C) => fn(pair[0], pair[1], z));
}

function combine4<A, B, C, D, R>(
  a: ReturnType<typeof behavior<A>>,
  b: ReturnType<typeof behavior<B>>,
  c: ReturnType<typeof behavior<C>>,
  d: ReturnType<typeof behavior<D>>,
  fn: (a: A, b: B, c: C, d: D) => R
): ReturnType<typeof behavior<R>> {
  return combine(
    combine(a, b, (x: A, y: B) => [x, y] as [A, B]),
    combine(c, d, (x: C, y: D) => [x, y] as [C, D]),
    (ab: [A, B], cd: [C, D]) => fn(ab[0], ab[1], cd[0], cd[1])
  );
}

// Validation result type
interface ValidationResult {
  valid: boolean;
  error: string;
}

// Form field with validation
interface FormField<T> {
  value: ReturnType<typeof behavior<T>>;
  touched: ReturnType<typeof behavior<boolean>>;
  validation: ReturnType<typeof behavior<ValidationResult>>;
}

// Validators
const validators = {
  required: (value: string): ValidationResult => ({
    valid: value.trim().length > 0,
    error: value.trim().length > 0 ? '' : 'This field is required',
  }),

  email: (value: string): ValidationResult => {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    const valid = emailRegex.test(value);
    return {
      valid,
      error: valid ? '' : 'Please enter a valid email address',
    };
  },

  minLength: (min: number) => (value: string): ValidationResult => ({
    valid: value.length >= min,
    error: value.length >= min ? '' : `Must be at least ${min} characters`,
  }),

  maxLength: (max: number) => (value: string): ValidationResult => ({
    valid: value.length <= max,
    error: value.length <= max ? '' : `Must be at most ${max} characters`,
  }),
};

// Password strength calculator
function calculatePasswordStrength(password: string): 'weak' | 'medium' | 'strong' {
  let score = 0;
  if (password.length >= 8) score++;
  if (password.length >= 12) score++;
  if (/[a-z]/.test(password) && /[A-Z]/.test(password)) score++;
  if (/\d/.test(password)) score++;
  if (/[^a-zA-Z0-9]/.test(password)) score++;

  if (score <= 2) return 'weak';
  if (score <= 3) return 'medium';
  return 'strong';
}

// Compose validators
function composeValidators(...fns: ((v: string) => ValidationResult)[]): (v: string) => ValidationResult {
  return (value: string) => {
    for (const fn of fns) {
      const result = fn(value);
      if (!result.valid) return result;
    }
    return { valid: true, error: '' };
  };
}

async function main() {
  // Initialize WASM module
  await init();

  // Form fields
  const name = behavior('');
  const nameTouched = behavior(false);

  const email = behavior('');
  const emailTouched = behavior(false);

  const password = behavior('');
  const passwordTouched = behavior(false);

  const bio = behavior('');
  const bioTouched = behavior(false);

  const submitted = behavior(false);
  const submitStatus = behavior<'idle' | 'success' | 'error'>('idle');

  // Validation rules
  const nameValidation = combine(name, nameTouched, (value: string, touched: boolean) => {
    if (!touched) return { valid: true, error: '' };
    return composeValidators(
      validators.required,
      validators.minLength(2),
      validators.maxLength(50)
    )(value);
  });

  const emailValidation = combine(email, emailTouched, (value: string, touched: boolean) => {
    if (!touched) return { valid: true, error: '' };
    return composeValidators(validators.required, validators.email)(value);
  });

  const passwordValidation = combine(password, passwordTouched, (value: string, touched: boolean) => {
    if (!touched) return { valid: true, error: '' };
    return composeValidators(
      validators.required,
      validators.minLength(8)
    )(value);
  });

  const bioValidation = combine(bio, bioTouched, (value: string, touched: boolean) => {
    if (!touched) return { valid: true, error: '' };
    return validators.maxLength(200)(value);
  });

  // Derived state
  const passwordStrength = password.map(calculatePasswordStrength);

  const bioCharCount = bio.map((text: string) => text.length);

  // Combine multiple validation behaviors
  const isFormValid = combine4(
    nameValidation,
    emailValidation,
    passwordValidation,
    bioValidation,
    (n: ValidationResult, e: ValidationResult, p: ValidationResult, b: ValidationResult) =>
      n.valid && e.valid && p.valid && b.valid
  );

  const allFieldsTouched = combine3(
    nameTouched,
    emailTouched,
    passwordTouched,
    (n: boolean, e: boolean, p: boolean) => n && e && p
  );

  const canSubmit = combine3(
    isFormValid,
    allFieldsTouched,
    submitted,
    (valid: boolean, touched: boolean, alreadySubmitted: boolean) =>
      valid && touched && !alreadySubmitted
  );

  // Input class based on validation state
  const inputClass = (validation: ReturnType<typeof behavior<ValidationResult>>, touched: ReturnType<typeof behavior<boolean>>) =>
    combine(validation, touched, (v: ValidationResult, t: boolean) => {
      if (!t) return '';
      return v.valid ? 'valid' : 'invalid';
    });

  // Event handlers
  const createInputHandler = (field: ReturnType<typeof behavior<string>>) => (e: Event) => {
    const target = e.target as HTMLInputElement;
    field.set(target.value);
  };

  const createBlurHandler = (touched: ReturnType<typeof behavior<boolean>>) => () => {
    touched.set(true);
  };

  const handleSubmit = (e: Event) => {
    e.preventDefault();

    // Touch all fields to show validation
    nameTouched.set(true);
    emailTouched.set(true);
    passwordTouched.set(true);
    bioTouched.set(true);

    // Check if form is valid
    const nameValid = composeValidators(validators.required, validators.minLength(2))(name.sample());
    const emailValid = composeValidators(validators.required, validators.email)(email.sample());
    const passwordValid = composeValidators(validators.required, validators.minLength(8))(password.sample());

    if (nameValid.valid && emailValid.valid && passwordValid.valid) {
      submitted.set(true);
      submitStatus.set('success');
      console.log('Form submitted:', {
        name: name.sample(),
        email: email.sample(),
        password: '***',
        bio: bio.sample(),
      });
    } else {
      submitStatus.set('error');
    }
  };

  // Status message
  const statusMessage = submitStatus.map((status: string) => {
    switch (status) {
      case 'success':
        return html`<div class="form-status success">Registration successful!</div>`;
      case 'error':
        return html`<div class="form-status error">Please fix the errors above.</div>`;
      default:
        return html`<span></span>`;
    }
  });

  // Password strength bar
  const strengthBar = passwordStrength.map((strength: string) =>
    html`<div class="password-strength-bar strength-${strength}"></div>`
  );

  // Submit button state
  const submitDisabled = canSubmit.map((can: boolean) => !can);

  // Main app template
  const app = html`
    <form class="form-app" onsubmit=${handleSubmit}>
      <h1>Registration Form</h1>

      <div class="form-group">
        <label for="name">Full Name</label>
        <input
          id="name"
          type="text"
          placeholder="John Doe"
          class=${inputClass(nameValidation, nameTouched)}
          oninput=${createInputHandler(name)}
          onblur=${createBlurHandler(nameTouched)}
        />
        <span class="error-message">${nameValidation.map((v: ValidationResult) => v.error)}</span>
      </div>

      <div class="form-group">
        <label for="email">Email</label>
        <input
          id="email"
          type="email"
          placeholder="john@example.com"
          class=${inputClass(emailValidation, emailTouched)}
          oninput=${createInputHandler(email)}
          onblur=${createBlurHandler(emailTouched)}
        />
        <span class="error-message">${emailValidation.map((v: ValidationResult) => v.error)}</span>
      </div>

      <div class="form-group">
        <label for="password">Password</label>
        <input
          id="password"
          type="password"
          placeholder="At least 8 characters"
          class=${inputClass(passwordValidation, passwordTouched)}
          oninput=${createInputHandler(password)}
          onblur=${createBlurHandler(passwordTouched)}
        />
        <div class="password-strength">
          ${strengthBar}
        </div>
        <span class="error-message">${passwordValidation.map((v: ValidationResult) => v.error)}</span>
      </div>

      <div class="form-group">
        <label for="bio">Bio (optional)</label>
        <textarea
          id="bio"
          placeholder="Tell us about yourself..."
          class=${inputClass(bioValidation, bioTouched)}
          oninput=${createInputHandler(bio)}
          onblur=${createBlurHandler(bioTouched)}
        ></textarea>
        <div class="field-info">
          <span class="error-message">${bioValidation.map((v: ValidationResult) => v.error)}</span>
          <span class="char-count">${bioCharCount}/200</span>
        </div>
      </div>

      <button type="submit" class="submit-btn" disabled=${submitDisabled}>
        Register
      </button>

      ${statusMessage}
    </form>
  `;

  // Mount to DOM
  mount(app, '#app');

  console.log('Cliffy TSX Forms initialized');
}

main().catch(console.error);
